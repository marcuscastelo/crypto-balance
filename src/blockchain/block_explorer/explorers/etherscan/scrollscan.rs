use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;

pub static SCROLLSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.scrollscan_api_key.clone(),
        base_url: "https://api.scrollscan.com/api".to_owned(),
        chain: LazyLock::new(|| &SCROLL),
    });
