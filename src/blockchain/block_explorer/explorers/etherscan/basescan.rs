use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;

pub static BASESCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.basescan_api_key.clone(),
        base_url: "https://api.basescan.org/api".to_string(),
        chain: LazyLock::new(|| &BASE),
    });
