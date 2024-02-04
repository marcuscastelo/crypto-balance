use crate::blockchain::prelude::*;
use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[async_trait]
pub trait BlockExplorer: Sync + Debug {
    async fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance;
    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> TokenBalance;
    async fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance>;
    fn get_chain(&self) -> &'static Chain;
}
