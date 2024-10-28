use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{blockchain::prelude::*, config::app_config::CONFIG};

pub static POLYGONSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.polygonscan_api_key.clone(),
        base_url: "https://api.polygonscan.com/api".to_string(),
        chain: LazyLock::new(|| &POLYGON),
    });
