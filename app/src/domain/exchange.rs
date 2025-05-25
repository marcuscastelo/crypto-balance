use thiserror::Error;

#[derive(Error, Debug)]
pub enum BalanceRepositoryError {
    #[error("Failed to fetch token names from repository")]
    FetchTokenNamesError,
}

pub trait BalanceRepository {
    async fn get_token_names(&self) -> error_stack::Result<Vec<String>, BalanceRepositoryError>;
    async fn update_balances(
        &self,
        target: &str,
        balances: &[f64],
    ) -> error_stack::Result<(), BalanceRepositoryError>;
}
