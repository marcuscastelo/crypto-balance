use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref POLYGONSCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Polygon".to_string(),
        api_key: CONFIG.polygonscan_api_key.clone(),
        base_url: "https://api.polygonscan.com/api".to_string(),
        native_token: Token::Native(NativeTokenName::MATIC),
    };
}
