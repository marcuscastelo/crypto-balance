use std::sync::LazyLock;

use config::Config;
use serde::Deserialize;
use serde_path_to_error::{Deserializer as PathDeserializer, Segment, Track};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub blockchain: super::blockchain_config::BlockchainConfig,
    pub sheets: super::sheets_config::SpreadsheetConfig,
    pub binance: super::binance_config::BinanceConfig,
    pub kraken: super::kraken_config::KrakenConfig,
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "Config".to_string());
    let config_result = Config::builder()
        .add_source(config::File::with_name(&config_path))
        .build();
    let config = match config_result {
        Ok(config) => config,
        Err(e) => match e {
            config::ConfigError::NotFound(property) => {
                panic!(
                    "[CONFIG ERROR] Missing property {:?} in config file: {}",
                    property, config_path
                );
            }
            _ => {
                panic!(
                    "[CONFIG ERROR] Error reading config file '{}': {:?}",
                    config_path, e
                );
            }
        },
    };
    let value = config
        .try_deserialize::<serde_json::Value>()
        .expect("Config to JSON failed");
    use serde::de::IntoDeserializer;
    let mut track = Track::new();
    let path_de = PathDeserializer::new(value.into_deserializer(), &mut track);
    match AppConfig::deserialize(path_de) {
        Ok(val) => val,
        Err(e) => {
            let path_str = track
                .path()
                .iter()
                .map(|seg| match seg {
                    Segment::Seq { index } => format!("[{}]", index),
                    Segment::Map { key } => format!(".{}", key),
                    Segment::Enum { variant } => format!("::{}", variant),
                    Segment::Unknown => String::from("<?>"),
                })
                .collect::<String>();
            panic!(
                "[CONFIG ERROR] Failed to deserialize config file '{}': {}\nField path: {}\nMake sure all required fields are present in the configuration file.",
                config_path, e, path_str.trim_start_matches('.')
            )
        }
    }
});
