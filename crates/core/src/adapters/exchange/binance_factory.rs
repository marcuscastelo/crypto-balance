use ::binance::{account::Account, api::Binance, config::Config};

use crate::adapters::config::binance_config::BinanceConfig;

pub struct BinanceAccountFactory {
    binance_config: BinanceConfig,
}

impl BinanceAccountFactory {
    pub fn new(binance_config: BinanceConfig) -> Self {
        Self { binance_config }
    }

    pub fn create(&self) -> Account {
        Binance::new_with_config(
            Some(self.binance_config.api_key.to_string()),
            Some(self.binance_config.secret_key.to_string()),
            &Config {
                rest_api_endpoint: "https://api.binance.com".into(),
                ws_endpoint: "wss://stream.binance.com:9443".into(),

                futures_rest_api_endpoint: "https://fapi.binance.com".into(),
                futures_ws_endpoint: "wss://fstream.binance.com".into(),

                recv_window: 50000,
                binance_us_api: false,

                timeout: None,
            },
        )
    }
}
