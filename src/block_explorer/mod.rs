use crate::token::{Token, TokenBalance};

pub mod arbiscan;
pub mod basescan;
pub mod etherscan;
pub mod lineascan;
pub mod optimistic_etherscan;
pub mod polygonscan;
pub mod prelude;
pub mod scrollscan;
pub mod zksync;
pub mod zora;

pub trait BlockExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance;
    fn get_native_token(&self) -> Token;
}
