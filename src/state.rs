use std::str::FromStr;
use std::sync::Arc;

use coins_bip32::xkeys::{self, Parent};
use color_eyre::eyre::{self, Result};
use ethers::core::k256::PublicKey as K256PublicKey;
use ethers::prelude::k256::elliptic_curve::sec1::ToEncodedPoint;
use ethers::providers::Middleware;
use ethers::types;
use ethers::types::transaction::eip2718::TypedTransaction;
use inquire::validator;
use ur_registry::crypto_hd_key::CryptoHDKey;
use ur_registry::crypto_key_path::CryptoKeyPath;
use ur_registry::ethereum;
use ur_registry::traits::{From, RegistryItem, To};

pub type EthersClient = ethers::providers::Provider<ethers::providers::Http>;

pub struct AppState<S> {
    config: crate::config::Config,
    term: console::Term,
    inner: S,
}

pub struct WithIntial;

impl AppState<WithIntial> {
    pub fn new(config: crate::config::Config) -> Self {
        let term = console::Term::stdout();
        let _ = term.clear_screen();
        term.set_title("ShekozWallet");
        Self {
            config,
            term,
            inner: WithIntial,
        }
    }

    /// Asks the user to select a network.
    pub fn select_network(self) -> Result<AppState<WithNetwork>> {
        let networks = self.config.networks.keys().collect::<Vec<_>>();
        let selected_network =
            inquire::Select::new("Choose your network", networks).prompt()?;
        let network = self.config.networks[selected_network].clone();
        Ok(AppState {
            config: self.config,
            term: self.term,
            inner: WithNetwork { network },
        })
    }
}

pub struct WithNetwork {
    network: crate::config::Network,
}

impl AppState<WithNetwork> {
    pub fn import_account(self) -> Result<AppState<WithAccount>> {
        self.term.write_line("Import your account using the QR")?;
        let content = crate::qrscanner::capture(&self.config)?.to_lowercase();
        // parse the message to know the type of the content.
        let (ty, _) = ur::ur::parse(&content)
            .map_err(|e| eyre::eyre!("Failed to parse the content: {}", e))?;
        assert_eq!(ty, CryptoHDKey::get_registry_type().get_type());

        let mut decoder = ur::Decoder::default();
        decoder
            .receive(&content)
            .map_err(|e| eyre::eyre!("Failed to decode the content: {}", e))?;
        assert!(decoder.complete());
        let message = decoder
            .message()
            .map_err(|e| eyre::eyre!("Failed to get the message: {}", e))?
            .ok_or_else(|| eyre::eyre!("No message found"))?;
        let hd_key = CryptoHDKey::from_bytes(message).map_err(|e| {
            eyre::eyre!("Failed to parse the message as a CryptoHDKey: {}", e)
        })?;
        let xpub = xkeys::XPub::from_str(&hd_key.get_bip32_key())?;
        // Derive many accounts and let the user select one.
        let mut accounts = Vec::new();
        let amount = 5;
        for i in 0..amount {
            let compressed_public_key = xpub
                .derive_path(
                    format!("0/{i}")
                        .parse::<coins_bip32::path::DerivationPath>()?,
                )?
                .to_bytes();
            let public_key =
                Self::decompress_public_key(compressed_public_key.into())?;
            let address = Self::public_key_to_address(&public_key);
            accounts.push((address, public_key));
        }
        let accounts_display = accounts
            .iter()
            .enumerate()
            .map(|(i, (address, _))| {
                if let Some(name) = hd_key.get_name() {
                    format!("{name} ({i}): {address}")
                } else {
                    format!("Account #{i}: {address}")
                }
            })
            .collect::<Vec<_>>();
        let selected_account =
            inquire::Select::new("Select an account", accounts_display.clone())
                .prompt()?;
        let selected_account_i = accounts_display
            .iter()
            .position(|a| a == &selected_account)
            .unwrap_or_default();
        let (address, _) = accounts[selected_account_i].clone();
        let crypto_key_path = CryptoKeyPath::from_path(
            format!("m/44'/60'/0'/0/{selected_account_i}"),
            hd_key.get_origin().and_then(|o| o.get_source_fingerprint()),
        ).map_err(|e| {
            eyre::eyre!(
                "Failed to create a CryptoKeyPath from the selected account: {}",
                e
            )
        })?;
        Ok(AppState {
            config: self.config,
            term: self.term,
            inner: WithAccount {
                network: self.inner.network,
                address,
                crypto_key_path,
            },
        })
    }

    /// Decompress the compressed public key and return the uncompressed public
    /// key. **Note:** it also removes the 0x04 prefix, so the result is the
    /// uncompressed public key without the prefix.
    fn decompress_public_key(compressed: Vec<u8>) -> Result<Vec<u8>> {
        let public_key = K256PublicKey::from_sec1_bytes(&compressed)?;
        let public_key =
            public_key.to_encoded_point(/* compress = */ false);
        let result = public_key.as_bytes();
        debug_assert_eq!(result[0], 0x04);
        if result.len() == 65 {
            // remove the 0x04 prefix
            Ok(result[1..].to_vec())
        } else {
            Ok(result.to_vec())
        }
    }

    /// Convert the uncompressed public key to an address.
    /// **Note:** this function assumes that the public key is uncompressed and
    /// doesn't have the 0x04 prefix.
    fn public_key_to_address(pub_key: &[u8]) -> types::Address {
        let hash = ethers::utils::keccak256(pub_key);
        types::Address::from_slice(&hash[12..])
    }
}

pub struct WithAccount {
    address: types::Address,
    crypto_key_path: CryptoKeyPath,
    network: crate::config::Network,
}

impl AppState<WithAccount> {
    pub fn ask_for_operation(self) -> Result<AppState<WithOperation>> {
        let network_native_token_symbol = &self.inner.network.currency_symbol;
        let operations = vec![
            "Transfer ERC20 Tokens".into(),
            format!("Transfer {network_native_token_symbol}"),
            "Sign a message".into(),
        ];
        let selected_operation =
            inquire::Select::new("Select an operation", operations).prompt()?;
        let operation = match selected_operation.as_str() {
            "Transfer ERC20 Tokens" => self.transfer_erc20_tokens(),
            "Sign a message" => self.sign_message(),
            _ => self.transfer_native_token(),
        }?;
        Ok(AppState {
            config: self.config,
            term: self.term,
            inner: operation,
        })
    }

    fn sign_message(&self) -> Result<WithOperation> {
        let message = inquire::Text::new("Message to sign").prompt()?;
        Ok(WithOperation::SignMessage(SignMessageOp {
            term: self.term.clone(),
            message: message.into(),
            address: self.inner.address,
            crypto_key_path: self.inner.crypto_key_path.clone(),
        }))
    }

    fn transfer_native_token(&self) -> Result<WithOperation> {
        let amount = inquire::Text::new("Amount to transfer")
            .with_validator(EtherAmountValidator)
            .prompt()
            .and_then(|s| {
                ethers::utils::parse_ether(s)
                    .map_err(|e| inquire::InquireError::Custom(e.into()))
            })?;
        let recipient = inquire::Text::new("Recipient address")
            .with_validator(AddressValidator)
            .with_autocomplete(AddressBookAutoComplete::new(
                self.config.contacts.clone(),
            ))
            .prompt()
            .and_then(|s| {
                // Check if it starts with 0x
                if s.starts_with("0x") {
                    s.parse::<types::Address>()
                } else {
                    // it must be a contact name then the address is in the
                    // format "name <address>" so we split
                    // on the space and take the second part
                    s.split(' ').nth(1).unwrap_or(&s).parse()
                }
                .map_err(|e| inquire::InquireError::Custom(e.into()))
            })?;
        Ok(WithOperation::NativeTransfer(NativeTransferOp {
            term: self.term.clone(),
            crypto_key_path: self.inner.crypto_key_path.clone(),
            to: recipient,
            amount,
            from: self.inner.address,
            client: self.create_ethers_client()?,
        }))
    }

    fn transfer_erc20_tokens(&self) -> Result<WithOperation> {
        let token = inquire::Text::new("Token address")
            .with_validator(AddressValidator)
            .with_autocomplete(Erc20AutoComplete::new(
                self.inner.network.erc20_tokens.clone(),
            ))
            .prompt()
            .and_then(|s| {
                // Check if it starts with 0x
                if s.starts_with("0x") {
                    s.parse::<types::Address>()
                } else {
                    // it must be a token name then the address is in the
                    // format "name <address>" so we split
                    // on the space and take the second part
                    s.split(' ').nth(1).unwrap_or(&s).parse()
                }
                .map_err(|e| inquire::InquireError::Custom(e.into()))
            })?;
        let amount = inquire::Text::new("Amount to transfer")
            .with_validator(EtherAmountValidator)
            .prompt()
            .and_then(|s| {
                ethers::utils::parse_ether(s)
                    .map_err(|e| inquire::InquireError::Custom(e.into()))
            })?;
        let recipient = inquire::Text::new("Recipient address")
            .with_validator(AddressValidator)
            .with_autocomplete(AddressBookAutoComplete::new(
                self.config.contacts.clone(),
            ))
            .prompt()
            .and_then(|s| {
                // Check if it starts with 0x
                if s.starts_with("0x") {
                    s.parse::<types::Address>()
                } else {
                    // it must be a contact name then the address is in the
                    // format "name <address>" so we split
                    // on the space and take the second part
                    s.split(' ').nth(1).unwrap_or(&s).parse()
                }
                .map_err(|e| inquire::InquireError::Custom(e.into()))
            })?;
        Ok(WithOperation::Erc20Transfer(Erc20TransferOp {
            term: self.term.clone(),
            crypto_key_path: self.inner.crypto_key_path.clone(),
            erc20_token: token,
            to: recipient,
            from: self.inner.address,
            amount,
            client: self.create_ethers_client()?,
        }))
    }

    fn create_ethers_client(&self) -> Result<EthersClient> {
        let reqwest_client = if let Some(ref proxy) = self.config.proxy {
            let proxy = reqwest::Proxy::all(proxy)?;
            reqwest::ClientBuilder::new()
                .proxy(proxy)
                .https_only(true)
                .build()?
        } else if self
            .inner
            .network
            .rpc_url
            .host_str()
            .map(|s| s == "localhost")
            .unwrap_or(false)
        {
            reqwest::ClientBuilder::new().build()?
        } else {
            reqwest::ClientBuilder::new().https_only(true).build()?
        };
        let http_provider = ethers::providers::Http::new_with_client(
            self.inner.network.rpc_url.clone(),
            reqwest_client,
        );
        let ethers_client = EthersClient::new(http_provider);
        Ok(ethers_client)
    }
}

pub struct SignMessageOp {
    term: console::Term,
    message: Vec<u8>,
    address: types::Address,
    crypto_key_path: CryptoKeyPath,
}

pub struct NativeTransferOp {
    term: console::Term,
    crypto_key_path: CryptoKeyPath,
    to: types::Address,
    from: types::Address,
    amount: ethers::types::U256,
    client: EthersClient,
}

pub struct Erc20TransferOp {
    term: console::Term,
    crypto_key_path: CryptoKeyPath,
    erc20_token: types::Address,
    to: types::Address,
    from: types::Address,
    amount: ethers::types::U256,
    client: EthersClient,
}

struct SignRequest<'a> {
    message: &'a [u8],
    address: types::Address,
    crypto_key_path: &'a CryptoKeyPath,
    data_type: ethereum::eth_sign_request::DataType,
}

pub enum WithOperation {
    SignMessage(SignMessageOp),
    NativeTransfer(NativeTransferOp),
    Erc20Transfer(Erc20TransferOp),
}

impl AppState<WithOperation> {
    pub async fn execute(self) -> Result<()> {
        match &self.inner {
            WithOperation::SignMessage(op) => self.sign_message(op),
            WithOperation::NativeTransfer(op) => {
                self.transfer_native_token(op).await
            }
            WithOperation::Erc20Transfer(op) => {
                self.transfer_erc20_tokens(op).await
            }
        }
    }

    fn sign_message(
        &self,
        SignMessageOp {
            term,
            message,
            address,
            crypto_key_path,
        }: &SignMessageOp,
    ) -> Result<()> {
        let signature = self.sign_and_get_signature(SignRequest {
            message,
            address: *address,
            crypto_key_path,
            data_type: ethereum::eth_sign_request::DataType::PersonalMessage,
        })?;
        term.write_line(&format!("Signature: {}", signature))?;
        Ok(())
    }

    async fn transfer_native_token(
        &self,
        NativeTransferOp {
            term,
            crypto_key_path,
            to,
            amount,
            from,
            client,
        }: &NativeTransferOp,
    ) -> Result<()> {
        let chain_id = client.get_chainid().await?;
        term.write_line("Fetching Balance...")?;
        let balance = client
            .get_balance(*from, None)
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch balance: {}", e))?;
        term.write_line(&format!(
            "Balance: {}",
            ethers::utils::format_ether(balance)
        ))?;
        term.write_line(&format!(
            "Sending {} to {}",
            ethers::utils::format_ether(*amount),
            to
        ))?;
        // fetch the nonce
        let nonce = client
            .get_transaction_count(*from, None)
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch nonce: {}", e))?;
        let mut tx = TypedTransaction::default();
        let tx = tx
            .set_to(*to)
            .set_value(*amount)
            .set_nonce(nonce)
            .set_chain_id(chain_id.as_u64());
        // calcculate the gas limit
        let gas_limit = client
            .estimate_gas(tx, None)
            .await
            .map_err(|e| eyre::eyre!("Failed to estimate gas: {}", e))?;
        // print the gas price
        let gas_price = client
            .get_gas_price()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch gas price: {}", e))?;
        term.write_line(&format!(
            "Gas Price: {} Gwei",
            ethers::utils::format_ether(gas_price),
        ))?;
        term.write_line(&format!("Gas Limit: {}", gas_limit))?;
        let tx = tx.set_gas(gas_limit).set_gas_price(gas_price);
        term.write_line(&format!(
            "Transaction: {}",
            serde_json::to_string_pretty(&tx)?
        ))?;
        // ask for confirmation
        let ok = inquire::Confirm::new("Do you want to send this transaction?")
            .prompt()?;
        if !ok {
            eyre::bail!("Aborted by user");
        }
        let signature = self.sign_and_get_signature(SignRequest {
            message: tx.rlp().as_ref(),
            address: *from,
            crypto_key_path,
            data_type: ethereum::eth_sign_request::DataType::TypedTransaction,
        })?;
        term.write_line(&format!("Signature: {}", signature))?;
        let tx_signed = tx.rlp_signed(&signature);
        let pending_tx = client.send_raw_transaction(tx_signed).await?;
        term.write_line(&format!("Transaction sent: {}", *pending_tx))?;
        let maybe_receipt = pending_tx.confirmations(1).await?;
        match maybe_receipt {
            Some(receipt) => {
                term.write_line(&format!(
                    "Transaction mined: {}",
                    receipt.transaction_hash
                ))?;
            }
            None => {
                eyre::bail!("Transaction not mined");
            }
        }
        Ok(())
    }

    async fn transfer_erc20_tokens(
        &self,
        Erc20TransferOp {
            term,
            crypto_key_path,
            to,
            amount,
            from,
            erc20_token,
            client,
        }: &Erc20TransferOp,
    ) -> Result<()> {
        let client = Arc::new(client.clone());
        let contract = crate::erc20::Erc20::new(*erc20_token, client);
        // Check user balance
        let balance = contract
            .balance_of(*from)
            .call()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch balance: {}", e))?;
        let decimals = contract
            .decimals()
            .call()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch decimals: {}", e))?;
        let token_symbol = contract
            .symbol()
            .call()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch symbol: {}", e))?;
        let formated = ethers::utils::format_units(balance, decimals as u32)?;
        term.write_line(&format!("Balance: {} {}", formated, token_symbol))?;
        let parsed_amount =
            ethers::utils::parse_units(amount.to_string(), decimals as u32)?;
        term.write_line(&format!(
            "Sending {} {} to {}",
            amount, token_symbol, to
        ))?;
        let nonce = contract
            .client()
            .get_transaction_count(*from, None)
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch nonce: {}", e))?;
        let transfer_tx = contract.transfer(*to, parsed_amount.into());
        let transfer_tx = transfer_tx.from(*from);
        // dry call.
        let gas_limit = transfer_tx
            .estimate_gas()
            .await
            .map_err(|e| eyre::eyre!("Failed to estimate gas: {}", e))?;
        let gas_price = contract
            .client()
            .get_gas_price()
            .await
            .map_err(|e| eyre::eyre!("Failed to fetch gas price: {}", e))?;
        term.write_line(&format!(
            "Gas Price: {} Gwei",
            ethers::utils::format_ether(gas_price),
        ))?;
        term.write_line(&format!("Gas Limit: {}", gas_limit))?;
        let mut transfer_tx = transfer_tx.gas(gas_limit).gas_price(gas_price);
        let tx_inner = transfer_tx.tx.set_nonce(nonce).clone();
        transfer_tx.tx = tx_inner;
        term.write_line(&format!(
            "Transaction: {}",
            serde_json::to_string_pretty(&transfer_tx.tx)?
        ))?;
        // dry call.
        let result = transfer_tx.call().await?;
        if !result {
            term.write_line("WARNING: Dry call failed. Transaction may fail.")?;
        }
        let tx_rlp = transfer_tx.tx.rlp();
        let signature = self.sign_and_get_signature(SignRequest {
            message: tx_rlp.as_ref(),
            address: *from,
            crypto_key_path,
            data_type: ethereum::eth_sign_request::DataType::TypedTransaction,
        })?;
        term.write_line(&format!("Signature: {}", signature))?;
        let tx_signed = transfer_tx.tx.rlp_signed(&signature);
        let client = contract.client();
        let pending_tx = client.send_raw_transaction(tx_signed).await?;
        term.write_line(&format!("Transaction sent: {}", *pending_tx))?;
        let maybe_receipt = pending_tx.confirmations(1).await?;
        match maybe_receipt {
            Some(receipt) => {
                term.write_line(&format!(
                    "Transaction mined: {}",
                    receipt.transaction_hash
                ))?;
            }
            None => {
                eyre::bail!("Transaction not mined");
            }
        }
        Ok(())
    }

    fn sign_and_get_signature(
        &self,
        SignRequest {
            message,
            address,
            crypto_key_path,
            data_type,
        }: SignRequest,
    ) -> Result<types::Signature> {
        let mut request = ethereum::eth_sign_request::EthSignRequest::default();
        request.set_derivation_path(crypto_key_path.clone());
        request.set_data_type(data_type);
        request.set_address(address.to_fixed_bytes().to_vec());
        request.set_sign_data(message.to_vec());
        request.set_request_id(Vec::new());
        let data = ur::encode(
            &request.to_bytes(),
            ethereum::eth_sign_request::EthSignRequest::get_registry_type()
                .get_type(),
        );
        crate::qrscanner::display_qr_code(&self.config, &data)?;
        let mut ready = false;
        while !ready {
            ready = inquire::Confirm::new(
                "Press Enter when the signature is ready to be scanned",
            )
            .with_default(true)
            .prompt()?;
        }
        println!();
        let content = crate::qrscanner::capture(&self.config)?.to_lowercase();
        // parse the message to know the type of the content.
        let (ty, _) = ur::ur::parse(&content)
            .map_err(|e| eyre::eyre!("Failed to parse the QR code: {}", e))?;
        assert_eq!(
            ty,
            ethereum::eth_signature::EthSignature::get_registry_type()
                .get_type()
        );

        let mut decoder = ur::Decoder::default();
        decoder
            .receive(&content)
            .map_err(|e| eyre::eyre!("Failed to decode the QR code: {}", e))?;
        assert!(decoder.complete());
        let message = decoder
            .message()
            .map_err(|e| eyre::eyre!("Failed to decode the QR code: {}", e))?
            .ok_or_else(|| eyre::eyre!("No message found"))?;
        let sig = ethereum::eth_signature::EthSignature::from_bytes(message)
            .map_err(|e| {
                eyre::eyre!(
                    "Failed to parse the message as a EthSignature: {}",
                    e
                )
            })?;
        let signature =
            types::Signature::try_from(sig.get_signature().as_slice())?;
        Ok(signature)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EtherAmountValidator;

impl validator::StringValidator for EtherAmountValidator {
    fn validate(
        &self,
        s: &str,
    ) -> Result<validator::Validation, inquire::CustomUserError> {
        match ethers::utils::parse_ether(s) {
            Ok(_) => Ok(validator::Validation::Valid),
            Err(e) => Ok(validator::Validation::Invalid(
                validator::ErrorMessage::Custom(e.to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AddressValidator;

impl validator::StringValidator for AddressValidator {
    fn validate(
        &self,
        s: &str,
    ) -> Result<validator::Validation, inquire::CustomUserError> {
        // Check if it starts with 0x
        let address = if s.starts_with("0x") {
            s
        } else {
            // it must be a contact name then the address is in the format "name
            // <address>" so we split on the space and take the
            // second part
            s.split(' ').nth(1).unwrap_or(s)
        };
        match types::Address::from_str(address) {
            Ok(_) => Ok(validator::Validation::Valid),
            Err(e) => Ok(validator::Validation::Invalid(
                validator::ErrorMessage::Custom(e.to_string()),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AddressBookAutoComplete {
    contacts: Vec<crate::config::Contact>,
}

impl AddressBookAutoComplete {
    fn new(contacts: Vec<crate::config::Contact>) -> Self {
        Self { contacts }
    }
}

impl inquire::Autocomplete for AddressBookAutoComplete {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> Result<Vec<String>, inquire::CustomUserError> {
        // search for the input as a contact name or address
        // and return the list of suggestions
        let suggestions = self
            .contacts
            .iter()
            .filter(|contact| {
                contact.name.to_lowercase().contains(input)
                    || contact
                        .address
                        .to_string()
                        .to_lowercase()
                        .contains(input)
            })
            .map(|contact| format!("{} {:?}", contact.name, contact.address))
            .collect();
        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError>
    {
        // we don't want to replace the input with a suggestion
        Ok(inquire::autocompletion::Replacement::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Erc20AutoComplete {
    erc20_tokens: Vec<crate::config::Erc20TokenConfig>,
}

impl Erc20AutoComplete {
    fn new(erc20_tokens: Vec<crate::config::Erc20TokenConfig>) -> Self {
        Self { erc20_tokens }
    }
}

impl inquire::Autocomplete for Erc20AutoComplete {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> Result<Vec<String>, inquire::CustomUserError> {
        // search for the input as a token name or address
        // and return the list of suggestions
        let suggestions = self
            .erc20_tokens
            .iter()
            .filter(|token| {
                token.name.to_lowercase().contains(input)
                    || token.address.to_string().to_lowercase().contains(input)
            })
            .map(|token| format!("{} {:?}", token.name, token.address))
            .collect();
        Ok(suggestions)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError>
    {
        // we don't want to replace the input with a suggestion
        Ok(inquire::autocompletion::Replacement::None)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn eth_sign_request() {
        let config = crate::config::Config {
            debug: true,
            ..Default::default()
        };
        let content =
            crate::qrscanner::capture(&config).unwrap().to_lowercase();
        let (ty, _) = ur::ur::parse(&content).unwrap();
        assert_eq!(
            ty,
            ethereum::eth_sign_request::EthSignRequest::get_registry_type()
                .get_type()
        );

        let mut decoder = ur::Decoder::default();
        decoder.receive(&content).unwrap();
        assert!(decoder.complete());
        let message = decoder.message().unwrap().unwrap();
        let req =
            ethereum::eth_sign_request::EthSignRequest::from_bytes(message)
                .unwrap();
        dbg!(&req);
        let data = req.get_sign_data();
        eprintln!("data: 0x{}", hex::encode(&data));
    }
}
