#[derive(serde::Deserialize, Debug, Clone)]
pub struct BybitConfig {
    pub api_key: Box<str>,
    pub secret_key: Box<str>,
}
