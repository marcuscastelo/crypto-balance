use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;

pub static ARBISCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.arbiscan_api_key.clone(),
        base_url: "https://api.arbiscan.io/api".to_string(),
        chain: LazyLock::new(|| &ARBITRUM),
    });
