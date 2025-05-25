use std::time::Duration;

use krakenrs::{KrakenCredentials, KrakenRestAPI, KrakenRestConfig};

use crate::infrastructure::config::kraken_config::KrakenConfig;

pub struct KrakenFactory {
    kraken_config: KrakenConfig,
}

impl KrakenFactory {
    pub fn new(kraken_config: KrakenConfig) -> Self {
        Self { kraken_config }
    }

    pub fn create(&self) -> KrakenRestAPI {
        let kc_config = KrakenRestConfig {
            timeout: Duration::new(30, 0),
            creds: KrakenCredentials {
                key: self.kraken_config.api_key.to_string(),
                secret: self.kraken_config.secret_key.to_string(),
            },
        };
        KrakenRestAPI::try_from(kc_config).expect("Should create kraken api")
    }
}
