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
    let erc20_balances = network.explorer.fetch_erc20_balances(evm_address);
    let mut balances = erc20_balances;
    balances.insert(network.native_token.to_owned(), native_balance);
    balances
}

fn main() {
    for network in NETWORKS.values() {
        println!("{} --------------", network.name);

        let network_balances = get_network_balance(network, &CONFIG.evm_address);

        for (token, balance) in network_balances {
            match token.as_ref() {
                Token::Native(token_name) => {
                    println!("   {} {:?}", balance.balance, token_name);
                }
                Token::ERC20(token_info) => {
                    println!(
                        "   {} {} ({})",
                        balance.balance, token_info.token_symbol, token_info.token_name
                    );
                }
            }
        }

        println!("   --------------");
        println!();
        println!();
    }
}
