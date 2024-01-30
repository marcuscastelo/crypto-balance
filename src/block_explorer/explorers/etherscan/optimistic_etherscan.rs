use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::OPTIMISM};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OPTIMISTIC_ETHERSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.optimistic_etherscan_api_key.clone(),
        base_url: "https://api-optimistic.etherscan.io/api".to_string(),
        network: LazyLock::new(|| &OPTIMISM),
    };
}
