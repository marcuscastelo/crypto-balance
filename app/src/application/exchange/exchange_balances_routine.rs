use std::{collections::HashMap, fmt, sync::Arc};

use tracing::instrument;

use crate::{
    application::sheets::spreadsheet::SpreadsheetUseCasesImpl,
    domain::routine::{Routine, RoutineError},
};

use super::use_cases::ExchangeUseCases;

pub struct ExchangeBalancesRoutine<'s> {
    routine_name: String,
    exchange: &'static dyn ExchangeUseCases,
    persistence: Arc<SpreadsheetUseCasesImpl<'s>>,
}

impl<'s> fmt::Debug for ExchangeBalancesRoutine<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExchangeBalancesRoutine")
            .field("routine_name", &self.routine_name)
            .finish()
    }
}

impl<'s> ExchangeBalancesRoutine<'s> {
    pub fn new(
        exchange: &'static dyn ExchangeUseCases,
        persistence: Arc<SpreadsheetUseCasesImpl<'s>>,
    ) -> Self {
        Self {
            routine_name: format!("{} Balances", exchange.exchange_name()),
            exchange,
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
impl<'s> Routine for ExchangeBalancesRoutine<'s> {
    fn name(&self) -> &str {
        self.routine_name.as_str()
    }

    #[instrument(skip(self), name = "ExchangeBalancesRoutine::run")]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("Binance: Running BinanceRoutine");

        tracing::trace!("{}: 📋 Listing all tokens from persistence", self.name());
        let token_names = self.persistence.get_token_names_from_spreadsheet().await;

        tracing::trace!("{}: ☁️  Getting balances from exchange", self.name());
        let balance_by_token = self.exchange.fetch_balances().await.map_err(|err| {
            tracing::error!("{}: ❌ Error fetching balances: {}", self.name(), err);
            RoutineError::routine_failure("Failed to fetch balances from exchange")
        })?;

        tracing::trace!("{}: 📊 Ordering balances", self.name());
        let token_balances = self.order_balances(token_names.as_slice(), &balance_by_token);

        tracing::trace!(
            "{}: 📝 Updating Binance balances on the spreadsheet",
            self.name()
        );
        self.persistence
            .update_balances_on_spreadsheet(
                self.exchange.spreadsheet_target(),
                token_balances.as_slice(),
            )
            .await;

        tracing::info!(
            "{}: ✅ Updated Binance balances on the spreadsheet",
            self.name()
        );

        Ok(())
    }
}
