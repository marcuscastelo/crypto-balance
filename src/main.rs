mod app_config;
mod block_explorer;
mod constants;

use crate::app_config::CONFIG;
use crate::block_explorer::etherscan::Etherscan;
use crate::block_explorer::prelude::*;

fn main() {
    let eth = Etherscan.fetch_balance(&CONFIG.evm_address);
    println!("Ethereum: {}", eth);
}
