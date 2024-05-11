use crate::prelude::*;

use crate::exchange::bybit::factory::BybitFactory;
use bybit_rs::bybit::account::Account;
use google_sheets4::api::ValueRange;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub struct UpdateBybitBalanceOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateBybitBalanceOnSheetsRoutine {
    async fn run(&self) {
        info!("Running UpdateBybitBalanceOnSheetsRoutine");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let mut bybit_api = BybitFactory::create();

        let token_names: Vec<String> = routines::sheets::SheetsGetTokenNamesRoutine.run().await;

        let response_value = bybit_api
            .get_wallet_balance(HashMap::from([(
                "accountType".to_owned(),
                "UNIFIED".to_owned(),
            )]))
            .await
            .expect("Should get wallet balance");

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

        // println!("Bybit response: {:#?}", result);

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

        // Write to the spreadsheet
        let mut token_balances = Vec::with_capacity(token_names.len());

        for token_name in &token_names {
            token_balances.push(balances.get(token_name).unwrap_or(&0.0));
        }

        spreadsheet_manager
            .write_named_range(
                ranges::balances::bybit::RW_AMOUNTS,
                ValueRange {
                    range: None,
                    major_dimension: None,
                    values: Some(
                        token_balances
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
