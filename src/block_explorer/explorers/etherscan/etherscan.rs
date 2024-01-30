use super::meta_etherscan::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ETHERSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.etherscan_api_key.clone(),
        base_url: "https://api.etherscan.io/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
