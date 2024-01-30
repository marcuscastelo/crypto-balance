use config::Config;
use lazy_static::lazy_static;

#[derive(serde::Deserialize, Debug)]
pub struct AppConfig {
    pub api_key: String,
    pub evm_address: String,
}

lazy_static! {
    pub static ref CONFIG: AppConfig = {
        let mut builder = Config::builder().add_source(config::File::with_name("Config"));
        builder.build().unwrap().try_deserialize().unwrap()
    };
}
