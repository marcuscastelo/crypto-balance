use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

pub struct FetchEvmChainBalancesRoutine;

impl FetchEvmChainBalancesRoutine {
    pub async fn run(&self, chain: &Chain, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        println!("Fetching balance for {}", chain.name);

        println!("Fetching native balance for {}", chain.name);
        let native_balance = chain.explorer.fetch_native_balance(evm_address).await;
        println!("Fetching ERC20 balances for {}", chain.name);
        let erc20_balances = chain.explorer.fetch_erc20_balances(evm_address).await;
        println!("Merging balances for {}", chain.name);
        let mut balances = erc20_balances;
        balances.insert(chain.native_token.to_owned(), native_balance);
        println!("Balances fetched for {}", chain.name);

        // Remove zero balances
        balances.retain(|_, balance| balance.balance > 0.0);
        balances
    }
}