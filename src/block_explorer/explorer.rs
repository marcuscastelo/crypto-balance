use crate::token::{Token, TokenBalance};

pub trait BlockExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance;
    fn get_native_token(&self) -> Token;
}
