use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OPTIMISTIC_ETHERSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.blockchain.optimistic_etherscan_api_key.clone(),
        base_url: "https://api-optimistic.etherscan.io/api".to_string(),
        chain: LazyLock::new(|| &OPTIMISM),
    };
}
