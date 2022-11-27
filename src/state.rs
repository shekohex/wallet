use ethers::types;

pub type EthersClient = ethers::providers::Provider<ethers::providers::Http>;

pub struct WalletState<S> {
    config: crate::config::Config,
    inner: S,
}

pub struct WithIntial;

impl WalletState<WithIntial> {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            config,
            inner: WithIntial,
        }
    }

    /// Asks the user to select a network.
    pub fn select_network(self) -> anyhow::Result<WalletState<WithNetwork>> {
        let networks = self.config.networks.keys().collect::<Vec<_>>();
        let selected_network =
            inquire::Select::new("Select a network", networks).prompt()?;
        let network = self.config.networks[selected_network].clone();
        Ok(WalletState {
            config: self.config,
            inner: WithNetwork { network },
        })
    }
}

pub struct WithNetwork {
    network: crate::config::Network,
}

pub struct WithAccount {
    account: types::Address,
    network: crate::config::Network,
}

pub struct WithNativeTransfer {
    account: types::Address,
    client: EthersClient,
    amount: types::U256,
    to: types::Address,
}

pub struct WithErc20Transfer {
    account: types::Address,
    client: EthersClient,
    amount: types::U256,
    to: types::Address,
    erc20_token: crate::config::Erc20TokenConfig,
}

impl WalletState<WithNetwork> {
    pub fn import_account(self) -> anyhow::Result<WalletState<WithAccount>> {
        // TODO: Read account from the QR code.
        todo!("read account from QR code")
    }
}

impl WalletState<WithAccount> {
    pub fn transfer_native(
        self,
    ) -> anyhow::Result<WalletState<WithNativeTransfer>> {
        todo!("transfer native")
    }

    pub fn transfer_erc20(
        self,
    ) -> anyhow::Result<WalletState<WithErc20Transfer>> {
        todo!("transfer erc20")
    }
}
