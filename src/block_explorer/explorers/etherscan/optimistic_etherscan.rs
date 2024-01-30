use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OPTIMISTIC_ETHERSCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Optimism".to_string(),
        api_key: CONFIG.optimistic_etherscan_api_key.clone(),
        base_url: "https://api-optimistic.etherscan.io/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
