use crate::domain::exchange::BalanceUpdateTarget;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeUseCasesError {
    #[error("Failed to fetch balances from {0}")]
    FetchBalancesError(&'static str),
    #[error("Internal error: {0}")]
    /// This error is used to wrap any internal errors that may occur in the exchange use cases.
    /// Some libs didn't implement the error trait, so we can't use them directly.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use error_stack::report;
    /// use crypto_balance_core::application::exchange::use_cases::ExchangeUseCasesError;
    /// # let error: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    /// let internal_error = error
    ///     .map_err(|e| { report!(ExchangeUseCasesError::InternalError(format!("{e:?}"))) });
    /// ```
    InternalError(String),
}

#[async_trait::async_trait]
pub trait ExchangeUseCases: Send + Sync {
    fn exchange_name(&self) -> &'static str;
    fn spreadsheet_target(&self) -> BalanceUpdateTarget;
    async fn fetch_balances(
        &self,
    ) -> error_stack::Result<HashMap<String, f64>, ExchangeUseCasesError>;
}
