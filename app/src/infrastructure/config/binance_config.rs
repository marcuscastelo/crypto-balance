#[derive(serde::Deserialize, Debug, Clone)]
pub struct BinanceConfig {
    pub api_key: Box<str>,
    pub secret_key: Box<str>,
}
