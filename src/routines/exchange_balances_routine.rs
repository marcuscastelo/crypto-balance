use std::collections::HashMap;

use indicatif::ProgressBar;

use crate::{
    cli::progress::{new_progress, ProgressBarExt},
    exchange::domain::exchange::ExchangeUseCases,
    sheets::data::spreadsheet::SpreadsheetUseCasesImpl,
};

use super::routine::{Routine, RoutineResult};

pub struct ExchangeBalancesRoutine {
    exchange: &'static dyn ExchangeUseCases,
    persistence: Box<SpreadsheetUseCasesImpl>,
}

impl ExchangeBalancesRoutine {
    pub fn new(exchange: &'static dyn ExchangeUseCases) -> Self {
        Self {
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
    fn name(&self) -> &'static str {
        "Binance Balances"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Binance: Running BinanceRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Binance: 📋 Listing all tokens from persistence");
        let token_names = self.persistence.get_token_names_from_spreadsheet().await;

        progress.trace("Binance: ☁️  Getting balances from exchange");
        let balance_by_token = self.exchange.get_balances().await;

        progress.trace("Binance: 📊 Ordering balances");
        let token_balances = self.order_balances(token_names.as_slice(), &balance_by_token);

        progress.trace("Binance: 📝 Updating Binance balances on the spreadsheet");
        self.persistence
            .update_binance_balances_on_spreadsheet(token_balances.as_slice())
            .await;

        progress.info("Binance: ✅ Updated Binance balances on the spreadsheet");

        Ok(())
    }
}
