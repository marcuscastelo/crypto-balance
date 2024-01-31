use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::LINEA};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LINEASCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.lineascan_api_key.clone(),
        base_url: "https://api.lineascan.build/api".to_string(),
        network: LazyLock::new(|| &LINEA),
    };
}