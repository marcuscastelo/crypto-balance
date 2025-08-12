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

impl BalanceUpdateTarget {
    pub fn range(&self) -> &'static str {
        match &self {
            BalanceUpdateTarget::Binance => {
                crate::domain::sheets::ranges::balances::binance::RW_AMOUNTS
            }
            BalanceUpdateTarget::Kraken => {
                crate::domain::sheets::ranges::balances::kraken::RW_AMOUNTS
            }
        }
    }
}

#[async_trait::async_trait]
pub trait BalanceRepository: Send + Sync {
    /// Fetches the token names from the repository. Those names are used to order the balances
    /// fetched from the exchange.
    async fn get_token_names(&self) -> error_stack::Result<Vec<String>, BalanceRepositoryError>;

    /// Updates the balances in the repository. The order of the balances must match the order of
    /// the token names fetched from the repository.
    async fn update_balances(
        &self,
        target: BalanceUpdateTarget,
        balances: &[f64],
    ) -> error_stack::Result<(), BalanceRepositoryError>;
}
