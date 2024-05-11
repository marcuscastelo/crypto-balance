use crate::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FetchCosmosChainBalancesRoutine;

impl FetchCosmosChainBalancesRoutine {
    pub async fn run(&self, chain: &Chain, cosmos_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        println!("Fetching balance for {}", chain.name);

        println!("Fetching native balance for {}", chain.name);
        let native_balance = chain.explorer.fetch_native_balance(cosmos_address).await;

        // println!("Fetching IBC balances for {}", chain.name);
        // let ibc_balances = chain.explorer.fetch_ibc_balances(cosmos_address).await;

        println!("Merging balances for {}", chain.name);

        let mut balances = HashMap::new();
        balances.insert(chain.native_token.to_owned(), native_balance);
        // balances.extend(ibc_balances);
        println!("Balances fetched for {}", chain.name);

        // Remove zero balances
        balances.retain(|_, balance| balance.balance > 0.0);
        balances
    }
}
