use std::sync::LazyLock;

use super::etherscan_implementation::EtherscanImplementation;
use crate::{app_config::CONFIG, chain::chains::LINEA};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LINEASCAN: EtherscanImplementation = EtherscanImplementation {
        api_key: CONFIG.blockchain.lineascan_api_key.clone(),
        base_url: "https://api.lineascan.build/api".to_string(),
        chain: LazyLock::new(|| &LINEA),
    };
}
