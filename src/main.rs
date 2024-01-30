mod app_config;
mod block_explorer;
mod constants;
mod token;

use crate::app_config::CONFIG;
use crate::block_explorer::basescan::Basescan;
use crate::block_explorer::etherscan::Etherscan;
use crate::block_explorer::lineascan::Lineascan;
use crate::block_explorer::prelude::*;
use crate::block_explorer::scrollscan::Scrollscan;
use crate::block_explorer::zksync::ZkSyncExplorer;

fn main() {
    let evm_address = &CONFIG.evm_address;
    println!("Ethereum: {:?}", Etherscan.fetch_balance(evm_address));
    println!("zkSync: {:?}", ZkSyncExplorer.fetch_balance(evm_address));
    println!("Scroll: {:?}", Scrollscan.fetch_balance(evm_address));
    println!("Linea: {:?}", Lineascan.fetch_balance(evm_address));
    println!("Base: {:?}", Basescan.fetch_balance(evm_address));
}
