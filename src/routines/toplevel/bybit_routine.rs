use crate::prelude::*;

use crate::exchange::bybit::factory::BybitFactory;
use bybit_rs::bybit::account::Account;
use cli::progress::{new_progress, ProgressBarExt};
use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;
use into::MyInto;
use serde::Deserialize;
use serde_json::Value;
use spreadsheet_manager::SpreadsheetManager;
use std::collections::HashMap;

pub struct BybitRoutine;

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceCoin {
    coin: String,
    equity: String,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceAccount {
    coin: Vec<BybitGetWalletBalanceCoin>,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceResult {
    list: Vec<BybitGetWalletBalanceAccount>,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceResponse {
    result: BybitGetWalletBalanceResult,
}

impl BybitRoutine {
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

    async fn get_bybit_balances(&self) -> HashMap<String, f64> {
        let bybit_account = BybitFactory::create();
        let response_value = bybit_account
            .get_wallet_balance(HashMap::from([(
                "accountType".to_owned(),
                "UNIFIED".to_owned(),
            )]))
            .await
            .expect("Should get wallet balance");

        let balances: BybitGetWalletBalanceResponse =
            serde_json::from_value(response_value).expect("Should deserialize response");

        let balances = HashMap::from(
            balances
                .result
                .list
                .iter()
                .map(|account| {
                    // All account coins as (coin, equity)
                    account
                        .coin
                        .iter()
                        .map(|coin| (coin.coin.clone(), coin.equity.parse::<f64>().unwrap()))
                })
                .flatten()
                .collect::<HashMap<_, _>>(),
        );
        balances
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

    async fn update_binance_balances_on_spreadsheet(&self, balances: &[f64]) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;
        spreadsheet_manager
            .write_named_range(
                ranges::balances::bybit::RW_AMOUNTS,
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
impl Routine for BybitRoutine {
    async fn run(&self) {
        log::info!("Running BybitRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Bybit: 📋 Listing all tokens from the spreadsheet");
        let token_names: Vec<String> = self.get_token_names_from_spreadsheet().await;

        progress.trace("Bybit: ☁️  Getting balances from Bybit");
        let balances = self.get_bybit_balances().await;

        progress.trace("Bybit: 📊 Ordering balances");
        let token_balances = self.order_balances(token_names.as_slice(), &balances).await;

        progress.trace("Bybit: 📝 Updating Bybit balances on the spreadsheet");
        self.update_binance_balances_on_spreadsheet(token_balances.as_slice())
            .await;

        progress.info("Bybit: ✅ Updated Bybit balances on the spreadsheet");
    }
}
