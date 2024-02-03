use crate::blockchain::prelude::*;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

pub trait BlockExplorer: Sync + Debug {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance;
    fn fetch_erc20_balance(&self, evm_address: &str, token_info: ERC20TokenInfo) -> TokenBalance;
    fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance>;
    fn get_chain(&self) -> &'static Chain;
}
