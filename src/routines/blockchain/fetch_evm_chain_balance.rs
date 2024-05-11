use std::{collections::HashMap, sync::Arc};

use error_stack::ResultExt;

use crate::prelude::*;

use self::block_explorer::explorer::FetchBalanceError;

pub struct FetchEvmChainBalancesRoutine;

impl FetchEvmChainBalancesRoutine {
    pub async fn run(
        &self,
        chain: &Chain,
        evm_address: &str,
    ) -> Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError> {
        info!("Fetching balance for {}", chain.name);

        info!("Fetching native balance for {}", chain.name);
        let native_balance = chain
            .explorer
            .fetch_native_balance(evm_address)
            .await
            .attach_printable_lazy(|| {
                format!(
                    "Failed to fetch native balance for chain '{}', address '{}'",
                    chain.name, evm_address
                )
            })?;

        info!("Fetching ERC20 balances for {}", chain.name);
        let erc20_balances = chain
            .explorer
            .fetch_erc20_balances(evm_address)
            .await
            .attach_printable_lazy(|| {
                format!(
                    "Failed to fetch ERC20 balances for chain '{}', address '{}'",
                    chain.name, evm_address
                )
            })?;

        info!("Merging balances for {}", chain.name);
        let mut balances = erc20_balances;
        balances.insert(chain.native_token.to_owned(), native_balance);

        info!("Balances fetched for {}", chain.name);

        // Remove zero balances
        balances.retain(|_, balance| balance.balance > 0.0);
        Ok(balances)
    }
}
