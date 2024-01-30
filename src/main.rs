mod app_config;
mod block_explorer;
mod constants;
mod token;

use std::collections::HashMap;

use crate::app_config::CONFIG;
use crate::block_explorer::prelude::*;
use crate::token::Token;

fn main() {
    main_native();
    // main_erc20();
}

fn main_native() {
    let mut network_explorer_map = HashMap::<&str, &'static dyn BlockExplorer>::new();
    network_explorer_map.insert("Ethereum", &*ETHERSCAN);
    network_explorer_map.insert("zkSync", &ZkSyncExplorer);
    network_explorer_map.insert("Scroll", &Scrollscan);
    network_explorer_map.insert("Linea", &Lineascan);
    network_explorer_map.insert("Base", &Basescan);
    network_explorer_map.insert("Arbitrum", &Arbiscan);
    network_explorer_map.insert("Optimism", &OptimisticEtherscan);
    network_explorer_map.insert("Arbitrum", &Arbiscan);
    network_explorer_map.insert("Zora", &ZoraExplorer);
    network_explorer_map.insert("Polygon", &Polygonscan);

    let evm_address = &CONFIG.evm_address;
    let mut totals: HashMap<Token, f64> = HashMap::<Token, f64>::new();
    for (network, explorer) in network_explorer_map {
        let balance = explorer.fetch_native_balance(evm_address);
        totals
            .entry(balance.token.clone())
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

fn main_erc20() {
    let ethereum_erc20_balances = ETHERSCAN.fetch_erc20_balances(&CONFIG.evm_address);
    println!(
        "Ethereum ERC-20 balances: {:#?}",
        ethereum_erc20_balances.values()
    );
}
