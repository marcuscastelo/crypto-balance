#[derive(serde::Deserialize, Debug, Clone)]
pub struct BinanceConfig {
    pub binance_api_key: Box<str>,
    pub binance_secret_key: Box<str>,
}
