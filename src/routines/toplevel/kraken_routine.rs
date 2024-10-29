use crate::{
    cli::progress::{new_progress, ProgressBarExt},
    config::app_config,
    exchange::kraken::factory::KrakenFactory,
    into::MyInto,
    ranges, routines,
    spreadsheet_manager::SpreadsheetManager,
    Routine,
};
use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;
use serde_json::Value;
use std::collections::HashMap;
pub struct KrakenRoutine;

#[allow(unused_imports)]
use num_traits::ToPrimitive;

impl KrakenRoutine {
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

    async fn get_kraken_balances(&self) -> HashMap<String, f64> {
        let kraken_api = KrakenFactory::create();
        kraken_api
            .get_account_balance()
            .await
            .unwrap()
            .into_iter()
            .map(|(symbol, amount)| {
                (
                    match symbol.as_str() {
                        "XXBT" => "BTC".to_string(),
                        "XETH" => "ETH".to_string(),
                        "XXRP" => "XRP".to_string(),
                        "ZUSD" => "USDT".to_string(),
                        _ => symbol,
                    },
                    amount.to_f64().expect("Should be convertible to f64"),
                )
            })
            .filter(|(_, amount)| *amount > 0.0)
            .collect::<HashMap<_, _>>()
    }

    async fn order_balances(
        &self,
        token_names: &[String],
        balances: &HashMap<String, f64>,
    ) -> Vec<f64> {
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in token_names {
            let token_balance = balances.get(token_name).unwrap_or(&0.0);
            token_balances.push(*token_balance);
        }
        token_balances
    }

    async fn update_kraken_balances_on_spreadsheet(&self, balances: &[f64]) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;
        spreadsheet_manager
            .write_named_range(
                ranges::balances::kraken::RW_AMOUNTS,
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
impl Routine for KrakenRoutine {
    async fn run(&self) {
        log::info!("Running UpdateKrakenBalanceOnSheetsRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Kraken: 📋 Listing all tokens from the spreadsheet");
        let token_names = self.get_token_names_from_spreadsheet().await;

        progress.trace("Kraken: ☁️  Getting balances from Kraken");
        let balance_by_token = self.get_kraken_balances().await;

        progress.trace("Kraken: 📊 Ordering balances");
        let token_balances = self
            .order_balances(token_names.as_slice(), &balance_by_token)
            .await;

        progress.trace("Kraken: 📝 Updating Kraken balances on the spreadsheet");
        self.update_kraken_balances_on_spreadsheet(token_balances.as_slice())
            .await;

        progress.info("Kraken: ✅ Updated Kraken balances on the spreadsheet");
    }
}
