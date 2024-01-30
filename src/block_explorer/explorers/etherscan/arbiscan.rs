use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::ARBITRUM};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ARBISCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.arbiscan_api_key.clone(),
        base_url: "https://api.arbiscan.io/api".to_string(),
        network: LazyLock::new(|| &ARBITRUM),
    };
}
