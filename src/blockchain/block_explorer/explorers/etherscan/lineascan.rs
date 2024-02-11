use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::blockchain::prelude::*;

pub static LINEASCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.lineascan_api_key.clone(),
        base_url: "https://api.lineascan.build/api".to_string(),
        chain: LazyLock::new(|| &LINEA),
    });
