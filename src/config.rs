use core::fmt;
use std::collections::HashMap;

use color_eyre::Result;
use ethers::types;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub networks: HashMap<String, Network>,
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub debug: bool,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Network {
    pub rpc_url: url::Url,
    pub explorer_url: Option<url::Url>,
    pub chain_id: types::U256,
    pub currency_symbol: String,
    #[serde(default)]
    pub erc20_tokens: Vec<Erc20TokenConfig>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Erc20TokenConfig {
    pub address: types::Address,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub address: types::Address,
}

impl fmt::Display for Erc20TokenConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}] ({})", self.symbol, self.name, self.address)
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}

impl Default for Config {
    fn default() -> Self {
        let proxy = std::env::var("https_proxy")
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .ok();
        let mut networks = HashMap::new();
        let eth_mainnet = Network {
            rpc_url: "https://api.securerpc.com/v1".parse().unwrap(),
            explorer_url: Some("https://etherscan.io".parse().unwrap()),
            chain_id: 1.into(),
            currency_symbol: "ETH".to_string(),
            erc20_tokens: vec![],
        };
        networks.insert("mainnet".to_string(), eth_mainnet);
        let polygon_mainnet = Network {
            rpc_url: "https://polygon-rpc.com".parse().unwrap(),
            explorer_url: Some("https://polygonscan.com".parse().unwrap()),
            chain_id: 137.into(),
            currency_symbol: "MATIC".to_string(),
            erc20_tokens: vec![
                Erc20TokenConfig {
                    name: "USD Coin (PoS)".to_string(),
                    symbol: "USDC".to_string(),
                    address: "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"
                        .parse()
                        .unwrap(),
                },
                Erc20TokenConfig {
                    name: "Binance-Peg BUSD Token".to_string(),
                    symbol: "BUSD".to_string(),
                    address: "0x9C9e5fD8bbc25984B178FdCE6117Defa39d2db39"
                        .parse()
                        .unwrap(),
                },
            ],
        };
        networks.insert("polygon".to_string(), polygon_mainnet);
        let bsc_mainnet = Network {
            rpc_url: "https://bscrpc.com".parse().unwrap(),
            explorer_url: Some("https://bscscan.com".parse().unwrap()),
            chain_id: 56.into(),
            currency_symbol: "BNB".to_string(),
            erc20_tokens: vec![],
        };
        networks.insert("bsc".to_string(), bsc_mainnet);
        let contacts = vec![
            // Null address
            Contact {
                name: "Burn".to_string(),
                address: types::Address::zero(),
            },
        ];
        Self {
            debug: false,
            networks,
            contacts,
            proxy,
        }
    }
}

/// Load the config `shekozwallet.json` from the current directory.
/// If the file does not exist, it will be created with the default config
/// values.
pub fn try_load_or_create_default() -> Result<Config> {
    let config_path = std::path::Path::new("shekozwallet.json");
    if config_path.exists() {
        let config_file = std::fs::File::open(config_path)?;
        let config = serde_json::from_reader(config_file)?;
        Ok(config)
    } else {
        let config = Config::default();
        let config_file = std::fs::File::create(config_path)?;
        serde_json::to_writer_pretty(config_file, &config)?;
        Ok(config)
    }
}