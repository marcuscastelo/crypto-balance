use crate::exchange::binance::factory::BinanceAccountFactory;
use crate::prelude::*;
use ::binance::account::Account as BinanceAccount;
use cli::progress::{new_progress, ProgressBarExt};
use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;
use into::MyInto;
use serde_json::Value;
use spreadsheet_manager::SpreadsheetManager;
use std::collections::HashMap;

pub struct BinanceRoutine;

impl BinanceRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }

    async fn get_token_names_from_spreadsheet(&self) -> Vec<String> {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }

    async fn get_binance_balances(&self) -> HashMap<String, f64> {
        let binance_account: BinanceAccount = BinanceAccountFactory::create();

        binance_account
            .get_account()
            .await
            .unwrap()
            .balances
            .into_iter()
            .filter(|x| x.free > 0.0)
            // Convert to Hashmap of token.asset, token.free
            .map(|token| (token.asset, token.free))
            .collect::<HashMap<_, _>>()
    }

    async fn order_balances(
        &self,
        token_names: &[String],
        balances: &HashMap<String, f64>,
    ) -> Vec<f64> {
        // Write to the spreadsheet
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in token_names {
            let token_balance = balances.get(token_name).unwrap_or(&0.0);
            token_balances.push(*token_balance);
        }
        token_balances
    }

    async fn update_binance_balances_on_spreadsheet(&self, balances: &[f64]) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;
        spreadsheet_manager
            .write_named_range(
                ranges::balances::binance::RW_AMOUNTS,
                // TODO: create Vec<T> to ValueRange conversion
                ValueRange {
                    range: None,
                    major_dimension: None,
                    values: Some(
                        balances
                            .into_iter()
                            .map(|balance| {
                                vec![Value::Number(
                                    serde_json::Number::from_f64(*balance).unwrap(),
                                )]
                            })
                            .collect::<Vec<_>>(),
                    ),
                },
            )
            .await
            .expect("Should write balances to the spreadsheet");
    }
}

#[async_trait::async_trait]
impl Routine for BinanceRoutine {
    async fn run(&self) {
        log::info!("Binance: Running UpdateBinanceBalanceOnSheetsRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Binance: 📋 Listing all tokens from the spreadsheet");
        let token_names = self.get_token_names_from_spreadsheet().await;

        progress.trace("Binance: ☁️ Getting balances from Binance");
        let balance_by_token = self.get_binance_balances().await;

        progress.trace("Binance: 📊 Ordering balances");
        let token_balances = self
            .order_balances(token_names.as_slice(), &balance_by_token)
            .await;

        progress.trace("Binance: 📝 Updating Binance balances on the spreadsheet");
        self.update_binance_balances_on_spreadsheet(token_balances.as_slice())
            .await;

        progress.info("Binance: ✅ Updated Binance balances on the spreadsheet");
    }
}
