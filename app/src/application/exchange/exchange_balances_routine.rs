use std::{collections::HashMap, fmt, sync::Arc};

use error_stack::ResultExt;
use tracing::instrument;

use crate::domain::{
    exchange::BalanceRepository,
    routine::{Routine, RoutineError},
};

use super::use_cases::ExchangeUseCases;

pub struct ExchangeBalancesRoutine<T: ExchangeUseCases> {
    routine_name: String,
    use_cases: T,
    persistence: Arc<dyn BalanceRepository>,
}

impl<T: ExchangeUseCases> fmt::Debug for ExchangeBalancesRoutine<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExchangeBalancesRoutine")
            .field("routine_name", &self.routine_name)
            .finish()
    }
}

impl<T: ExchangeUseCases> ExchangeBalancesRoutine<T> {
    pub fn new(use_cases: T, persistence: Arc<dyn BalanceRepository>) -> Self {
        Self {
            routine_name: format!("{} Balances", use_cases.exchange_name()),
            use_cases,
            persistence,
        }
    }

    #[instrument]
    fn order_balances(&self, token_names: &[String], balances: &HashMap<String, f64>) -> Vec<f64> {
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in token_names {
            let token_balance = balances.get(token_name).unwrap_or(&0.0);
            token_balances.push(*token_balance);
        }
        token_balances
    }
}

#[async_trait::async_trait]
impl<T: ExchangeUseCases> Routine for ExchangeBalancesRoutine<T> {
    fn name(&self) -> &str {
        self.routine_name.as_str()
    }

    #[instrument(skip(self), name = "ExchangeBalancesRoutine::run")]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("{} started", self.name());

        tracing::trace!("{}: 📋 Listing all tokens from persistence", self.name());
        let token_names = self.persistence.get_token_names().await.change_context(
            RoutineError::routine_failure("Failed to get token names from persistence"),
        )?;

        tracing::trace!("{}: ☁️  Getting balances from exchange", self.name());
        let balance_by_token =
            self.use_cases
                .fetch_balances()
                .await
                .change_context(RoutineError::routine_failure(
                    "Failed to fetch balances from exchange",
                ))?;

        tracing::trace!("{}: 📊 Ordering balances", self.name());
        let token_balances = self.order_balances(token_names.as_slice(), &balance_by_token);

        tracing::trace!("{}: 📝 Updating balances on the spreadsheet", self.name());
        self.persistence
            .update_balances(
                self.use_cases.spreadsheet_target(),
                token_balances.as_slice(),
            )
            .await
            .change_context(RoutineError::routine_failure(
                "Failed to update balances in persistence",
            ))?;

        tracing::info!(
            "{}: ✅ Updated {} balances on the spreadsheet",
            self.name(),
            token_balances.len()
        );

        Ok(())
    }
}
