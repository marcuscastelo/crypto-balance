use std::sync::LazyLock;

use config::Config;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub blockchain: super::blockchain_config::BlockchainConfig,
    pub sheets: super::sheets_config::SpreadsheetConfig,
    pub binance: super::binance_config::BinanceConfig,
    pub kraken: super::kraken_config::KrakenConfig,
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    match Config::builder()
        .add_source(config::File::with_name("Config"))
        .build()
    {
        Ok(config) => config,
        Err(e) => match e {
            config::ConfigError::NotFound(property) => {
                panic!("Missing config property: {:?}", property);
            }
            _ => {
                panic!("Error reading config file: {:?}", e);
            }
        },
    }
    .try_deserialize()
    .expect("Should deserialize built config into struct")
});
