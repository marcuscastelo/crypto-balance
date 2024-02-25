use std::sync::LazyLock;

use config::Config;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub blockchain: BlockchainConfig,
    pub sheets: crate::sheets::config::SpreadsheetConfig,
    pub binance: crate::exchange::binance::config::BinanceConfig,
    pub kraken: crate::exchange::kraken::config::KrakenConfig,
    pub coingecko: crate::price::coingecko::config::CoingeckoConfig,
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
    pub evm: EvmBlockchainConfig,
    pub cosmos: CosmosBlockchainConfig,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct EvmBlockchainConfig {
    pub address: Box<str>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CosmosBlockchainConfig {
    pub cosmos_address: Box<str>,
    pub osmosis_address: Box<str>,
    pub celestia_address: Box<str>,
    pub injective_address: Box<str>,
    pub kujira_address: Box<str>,
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name("Config"))
        .build()
        .expect("Should build config from file")
        .try_deserialize()
        .expect("Should deserialize built config into struct")
});
