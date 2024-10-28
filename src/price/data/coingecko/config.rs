#[derive(serde::Deserialize, Debug, Clone)]
pub struct CoingeckoConfig {
    pub api_key: Box<str>,
}
