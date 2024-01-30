use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, network::networks::ETHEREUM};

pub static ETHERSCAN: LazyLock<EtherscanImplementation> =
    LazyLock::new(|| EtherscanImplementation {
        api_key: CONFIG.etherscan_api_key.clone(),
        base_url: "https://api.etherscan.io/api".to_string(),
        network: LazyLock::new(|| &ETHEREUM),
    });
