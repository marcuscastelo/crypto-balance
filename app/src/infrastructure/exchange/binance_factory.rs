use ::binance::{account::Account, api::Binance, config::Config};

use crate::infrastructure::config::app_config::CONFIG;

pub struct BinanceAccountFactory;

impl BinanceAccountFactory {
    pub fn create() -> Account {
        Binance::new_with_config(
            Some(CONFIG.binance.api_key.to_string()),
            Some(CONFIG.binance.secret_key.to_string()),
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
