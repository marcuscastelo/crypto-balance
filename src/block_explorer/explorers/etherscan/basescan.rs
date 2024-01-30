use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BASESCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Base".to_string(),
        api_key: CONFIG.basescan_api_key.clone(),
        base_url: "https://api.basescan.org/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
