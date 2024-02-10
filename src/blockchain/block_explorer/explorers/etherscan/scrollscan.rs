use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCROLLSCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.blockchain.scrollscan_api_key.clone(),
        base_url: "https://api.scrollscan.com/api".to_owned(),
        chain: LazyLock::new(|| &SCROLL),
    };
}
