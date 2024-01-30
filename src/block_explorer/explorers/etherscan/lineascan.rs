use super::etherscan_implementation::EtherscanImplementation;
use crate::{
    app_config::CONFIG,
    token::{NativeTokenName, Token},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LINEASCAN: EtherscanImplementation = EtherscanImplementation {
        network_name: "Linea".to_string(),
        api_key: CONFIG.lineascan_api_key.clone(),
        base_url: "https://api.lineascan.build/api".to_string(),
        native_token: Token::Native(NativeTokenName::ETH),
    };
}
