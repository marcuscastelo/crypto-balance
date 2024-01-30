use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::SCROLL};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCROLLSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.scrollscan_api_key.clone(),
        base_url: "https://api.scrollscan.com/api".to_string(),
        network: LazyLock::new(|| &SCROLL),
    };
}
