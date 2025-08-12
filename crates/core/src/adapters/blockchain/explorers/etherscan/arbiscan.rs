use std::sync::LazyLock;

use crate::adapters::{blockchain::chains::ARBITRUM, config::app_config::CONFIG};

use super::etherscan_implementation::EtherscanImplementation;

pub static ARBISCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.arbiscan_api_key.clone(),
        base_url: "https://api.arbiscan.io/api".to_string(),
        chain: LazyLock::new(|| &ARBITRUM),
    });
