use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, chain::chains::ETHEREUM};

pub static ETHERSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.blockchain.etherscan_api_key.clone(),
        base_url: "https://api.etherscan.io/api".to_string(),
        chain: LazyLock::new(|| &ETHEREUM),
    });
