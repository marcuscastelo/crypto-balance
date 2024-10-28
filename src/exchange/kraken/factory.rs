use std::time::Duration;

use krakenrs::{KrakenCredentials, KrakenRestAPI, KrakenRestConfig};

use crate::config::app_config::CONFIG;

pub struct KrakenFactory;

impl KrakenFactory {
    pub fn create() -> KrakenRestAPI {
        let kc_config = KrakenRestConfig {
            timeout: Duration::new(30, 0),
            creds: KrakenCredentials {
                key: CONFIG.kraken.api_key.to_string(),
                secret: CONFIG.kraken.secret_key.to_string(),
            },
        };
        KrakenRestAPI::try_from(kc_config).expect("Should create kraken api")
    }
}
