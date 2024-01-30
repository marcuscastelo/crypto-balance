#![feature(lazy_cell)]

mod app_config;
mod block_explorer;
mod constants;
mod network;
mod prelude;
mod token;

use std::collections::HashMap;
use std::sync::Arc;

use crate::prelude::*;

fn get_network_balance(network: &Network, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
    let native_balance = network.explorer.fetch_native_balance(evm_address);
    dbg!(&native_balance);
    let erc20_balances = network.explorer.fetch_erc20_balances(evm_address);
    dbg!(&erc20_balances);
    let mut balances = erc20_balances;
    balances.insert(network.native_token.to_owned(), native_balance);
    balances
}

fn main() {
    for network in NETWORKS.values() {
        dbg!(network);
        let network_balances = get_network_balance(network, &CONFIG.evm_address);
        println!(
            "{:?} balances: {:#?}",
            network.name,
            network_balances.values()
        );
    }
}
