use std::sync::LazyLock;

use crate::adapters::{blockchain::chains::POLYGON, config::app_config::CONFIG};

use super::etherscan_implementation::EtherscanImplementation;

pub static POLYGONSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.polygonscan_api_key.clone(),
        base_url: "https://api.polygonscan.com/api".to_string(),
        chain: LazyLock::new(|| &POLYGON),
    });
