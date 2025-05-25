use async_trait::async_trait;
use core::fmt;
use error_stack::{Context, Result};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use super::{
    chain::Chain,
    token::{ERC20TokenInfo, Token},
    token_balance::TokenBalance,
};

#[derive(Debug)]
pub enum FetchBalanceError {
    ReqwestError,
    DataFormatError,
}

impl fmt::Display for FetchBalanceError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

// It's also possible to implement `Error` instead.
impl Context for FetchBalanceError {}

#[async_trait]
pub trait BlockExplorer: Sync + Debug {
    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_native_balance(&self, address: &str) -> Result<TokenBalance, FetchBalanceError>;

    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> Result<TokenBalance, FetchBalanceError>;

    #[deprecated(note = "Use fetch_balances to get all balances")]
    async fn fetch_erc20_balances(
        &self,
        address: &str,
    ) -> Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError>;

    fn get_chain(&self) -> &'static Chain;
}
