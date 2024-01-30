use config::Config;
use lazy_static::lazy_static;

#[derive(serde::Deserialize, Debug)]
pub struct AppConfig {
    pub etherscan_api_key: String,
    pub scrollscan_api_key: String,
    pub lineascan_api_key: String,
    pub basescan_api_key: String,
    pub arbiscan_api_key: String,
    pub optimistic_etherscan_api_key: String,
    pub evm_address: String,
}

lazy_static! {
    pub static ref CONFIG: AppConfig = {
        Config::builder()
            .add_source(config::File::with_name("Config"))
            .build()
            .expect("Should build config from file")
            .try_deserialize()
            .expect("Should deserialize built config into struct")
    };
}
