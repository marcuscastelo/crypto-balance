#[derive(serde::Deserialize, Debug, Clone)]
pub struct ScrapingConfig {
    pub sonar_watch: SonarWatchConfig,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SonarWatchConfig {
    pub auth_token: Box<str>,
    pub turnstile_token: Box<str>,
}
