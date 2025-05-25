use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceRepositoryError {
    #[error("Failed to fetch token names from repository")]
    FetchTokenNamesError,
    #[error("Failed to update balances in repository")]
    UpdateBalancesError,
}

#[derive(Debug, Clone, Copy)]
pub enum BalanceUpdateTarget {
    Binance,
    Kraken,
}

#[async_trait::async_trait]
pub trait BalanceRepository: Send + Sync {
    async fn get_token_names(&self) -> error_stack::Result<Vec<String>, BalanceRepositoryError>;
    async fn update_balances(
        &self,
        target: BalanceUpdateTarget,
        balances: &[f64],
    ) -> error_stack::Result<(), BalanceRepositoryError>;
}
