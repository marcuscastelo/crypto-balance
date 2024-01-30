use std::collections::HashMap;

use crate::token::{ERC20TokenInfo, Token, TokenBalance};

pub trait BlockExplorer {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance;
    fn fetch_erc20_balance(&self, evm_address: &str, token_info: &ERC20TokenInfo) -> TokenBalance;
    fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Token, TokenBalance>;
    fn get_native_token(&self) -> Token;
}
