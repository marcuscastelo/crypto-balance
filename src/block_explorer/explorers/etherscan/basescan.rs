use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::BASE};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BASESCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.basescan_api_key.clone(),
        base_url: "https://api.basescan.org/api".to_string(),
        network: LazyLock::new(|| &BASE),
    };
}
