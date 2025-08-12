use async_trait::async_trait;
use error_stack::Result;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use thiserror::Error;

use super::{
    chain::Chain,
    token::{ERC20TokenInfo, Token},
    token_balance::TokenBalance,
};

#[derive(Error, Debug)]
pub enum FetchBalanceError {
    #[error("Failed to fetch balance")]
    ApiRequestError,
    #[error("Failed to parse response")]
    ResponseParsingError,
}

#[async_trait]
pub trait BlockExplorer: Sync + Debug {
    async fn fetch_native_balance(&self, address: &str) -> Result<TokenBalance, FetchBalanceError>;

    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> Result<TokenBalance, FetchBalanceError>;

    async fn fetch_erc20_balances(
        &self,
        address: &str,
    ) -> Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError>;

    fn chain(&self) -> &'static Chain;
}
