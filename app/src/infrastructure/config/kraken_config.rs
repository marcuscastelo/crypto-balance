#[derive(serde::Deserialize, Debug, Clone)]
pub struct KrakenConfig {
    pub api_key: Box<str>,
    pub secret_key: Box<str>,
}
