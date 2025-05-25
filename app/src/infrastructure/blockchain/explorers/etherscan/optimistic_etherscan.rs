use std::sync::LazyLock;

use crate::infrastructure::{blockchain::chains::OPTIMISM, config::app_config::CONFIG};

use super::etherscan_implementation::EtherscanImplementation;

pub static OPTIMISTIC_ETHERSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.optimistic_etherscan_api_key.clone(),
        base_url: "https://api-optimistic.etherscan.io/api".to_string(),
        chain: LazyLock::new(|| &OPTIMISM),
    });
