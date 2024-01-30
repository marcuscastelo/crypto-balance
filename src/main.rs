mod app_config;
mod block_explorer;
mod constants;
mod token;

use std::collections::HashMap;

use crate::app_config::CONFIG;
use crate::block_explorer::prelude::*;
use crate::token::Token;

fn main() {
    let mut network_explorer_map = HashMap::<&str, Box<dyn BlockExplorer>>::new();
    network_explorer_map.insert("Ethereum", Box::new(Etherscan));
    network_explorer_map.insert("zkSync", Box::new(ZkSyncExplorer));
    network_explorer_map.insert("Scroll", Box::new(Scrollscan));
    network_explorer_map.insert("Linea", Box::new(Lineascan));
    network_explorer_map.insert("Base", Box::new(Basescan));
    network_explorer_map.insert("Arbitrum", Box::new(Arbiscan));
    network_explorer_map.insert("Optimism", Box::new(OptimisticEtherscan));
    network_explorer_map.insert("Arbitrum", Box::new(Arbiscan));
    network_explorer_map.insert("Zora", Box::new(ZoraExplorer));
    network_explorer_map.insert("Polygon", Box::new(Polygonscan));

    let evm_address = &CONFIG.evm_address;
    let mut totals: HashMap<Token, f64> = HashMap::<Token, f64>::new();
    for (network, explorer) in network_explorer_map {
        let balance = explorer.fetch_balance(evm_address);
        totals
            .entry(balance.token)
            .and_modify(|e| *e += balance.balance)
            .or_insert(balance.balance);
        println!("{}: {:?} {:?}", network, balance, balance.token);
    }

    for (token, balance) in totals {
        println!("Total {:?}: {}", token, balance);
    }

    // TODO: Fetch all ERC-20 balances
    // TODO: Create EtherscanBasedExplorer to reduce code duplication
}
