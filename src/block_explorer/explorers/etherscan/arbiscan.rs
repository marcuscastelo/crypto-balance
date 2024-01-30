use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARBISCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Arbitrum".to_string(),
        api_key: CONFIG.arbiscan_api_key.clone(),
        base_url: "https://api.arbiscan.io/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
