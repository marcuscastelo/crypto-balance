use config::Config;
use lazy_static::lazy_static;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub blockchain: BlockchainConfig,
    pub sheets: crate::sheets::config::SpreadsheetConfig,
    pub binance: crate::exchange::binance_config::BinanceConfig,
}

// TODO: move to blockchain module
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BlockchainConfig {
    pub etherscan_api_key: Box<str>,
    pub scrollscan_api_key: Box<str>,
    pub lineascan_api_key: Box<str>,
    pub basescan_api_key: Box<str>,
    pub arbiscan_api_key: Box<str>,
    pub optimistic_etherscan_api_key: Box<str>,
    pub polygonscan_api_key: Box<str>,
    pub evm_address: String,
}

lazy_static! {
    pub static ref CONFIG: AppConfig = {
        Config::builder()
            .add_source(config::File::with_name("Config"))
            .build()
            .expect("Should build config from file")
            .try_deserialize()
            .expect("Should deserialize built config into struct")
    };
}
