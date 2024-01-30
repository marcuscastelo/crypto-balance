use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::POLYGON};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref POLYGONSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.polygonscan_api_key.clone(),
        base_url: "https://api.polygonscan.com/api".to_string(),
        network: LazyLock::new(|| &POLYGON),
    };
}
