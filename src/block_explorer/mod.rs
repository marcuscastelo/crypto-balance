use crate::token::{Token, TokenBalance};

pub mod arbiscan;
pub mod basescan;
pub mod etherscan;
pub mod lineascan;
pub mod optimistic_etherscan;
pub mod prelude;
pub mod scrollscan;
pub mod zksync;

pub trait BlockExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance;
    fn get_native_token(&self) -> Token;
}
