use crate::token::{Token, TokenBalance};

pub mod etherscan;
pub mod prelude;
pub mod scrollscan;
pub mod zksync;

pub trait BlockExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance;
    fn get_native_token(&self) -> Token;
}
