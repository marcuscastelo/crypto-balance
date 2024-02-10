use crate::blockchain::prelude::*;
use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[async_trait]
pub trait BlockExplorer: Sync + Debug {
    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_native_balance(&self, address: &str) -> TokenBalance;
    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> TokenBalance;
    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_erc20_balances(&self, address: &str) -> HashMap<Arc<Token>, TokenBalance>;
    fn get_chain(&self) -> &'static Chain;
}
