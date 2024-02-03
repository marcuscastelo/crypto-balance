use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, chain::chains::POLYGON};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref POLYGONSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.blockchain.polygonscan_api_key.clone(),
        base_url: "https://api.polygonscan.com/api".to_string(),
        chain: LazyLock::new(|| &POLYGON),
    };
}
