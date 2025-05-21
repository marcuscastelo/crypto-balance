use std::collections::HashMap;

use indicatif::ProgressBar;
use tracing::instrument;

use crate::{
    cli::progress::{new_progress, ProgressBarExt},
    exchange::domain::exchange::ExchangeUseCases,
    routines::routine::RoutineFailureInfo,
    sheets::data::spreadsheet::{BalanceUpdateTarget, SpreadsheetUseCasesImpl},
};

use super::routine::{Routine, RoutineResult};

pub struct ExchangeBalancesRoutine {
    routine_name: String,
    exchange: &'static dyn ExchangeUseCases,
    persistence: Box<SpreadsheetUseCasesImpl>,
}

impl ExchangeBalancesRoutine {
    pub fn new(exchange: &'static dyn ExchangeUseCases) -> Self {
        Self {
            routine_name: format!("{} Balances", exchange.exchange_name()),
            exchange,
            persistence: Box::new(SpreadsheetUseCasesImpl),
        }
    }

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
impl Routine for ExchangeBalancesRoutine {
    fn name(&self) -> &str {
        self.routine_name.as_str()
    }

    #[instrument(skip(self))]
    async fn run(&self) -> RoutineResult {
        tracing::info!("Binance: Running BinanceRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace(format!(
            "{}: 📋 Listing all tokens from persistence",
            self.name()
        ));
        let token_names = self.persistence.get_token_names_from_spreadsheet().await;

        progress.trace(format!(
            "{}: ☁️  Getting balances from exchange",
            self.name()
        ));
        let balance_by_token = self.exchange.fetch_balances().await;

        let balance_by_token = match balance_by_token {
            Ok(balances) => balances,
            Err(err) => {
                progress.error(format!(
                    "{}: ❌ Error fetching balances: {}",
                    self.name(),
                    err
                ));
                return Err(RoutineFailureInfo::new(format!(
                    "Error fetching balances: {}",
                    err
                )));
            }
        };

        progress.trace(format!("{}: 📊 Ordering balances", self.name()));
        let token_balances = self.order_balances(token_names.as_slice(), &balance_by_token);

        progress.trace(format!(
            "{}: 📝 Updating Binance balances on the spreadsheet",
            self.name()
        ));
        self.persistence
            .update_balances_on_spreadsheet(
                self.exchange.spreadsheet_target(),
                token_balances.as_slice(),
            )
            .await;

        progress.info(format!(
            "{}: ✅ Updated Binance balances on the spreadsheet",
            self.name()
        ));

        Ok(())
    }
}
