use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCROLLSCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Scroll".to_string(),
        api_key: CONFIG.scrollscan_api_key.clone(),
        base_url: "https://api.scrollscan.com/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
