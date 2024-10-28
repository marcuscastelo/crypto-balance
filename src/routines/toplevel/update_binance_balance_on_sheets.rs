use crate::exchange::binance::factory::BinanceAccountFactory;
use crate::prelude::*;
use ::binance::account::Account as BinanceAccount;
use google_sheets4::api::ValueRange;
use serde_json::Value;
use std::collections::HashMap;

pub struct UpdateBinanceBalanceOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateBinanceBalanceOnSheetsRoutine {
    async fn run(&self) {
        log::info!("Running UpdateBinanceBalanceOnSheetsRoutine");

        let spreadsheet_manager =
            SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await;

        let binance_account: BinanceAccount = BinanceAccountFactory::create();

        let token_names: Vec<String> = routines::sheets::SheetsGetTokenNamesRoutine.run().await;

        let balances = binance_account
            .get_account()
            .await
            .unwrap()
            .balances
            .into_iter()
            .filter(|x| x.free > 0.0)
            // Convert to Hashmap of token.asset, token.free
            .map(|token| (token.asset, token.free))
            .collect::<HashMap<_, _>>();

        log::info!("Binance Balances: {:?}", balances);

        // Write to the spreadsheet
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in &token_names {
            token_balances.push(balances.get(token_name).unwrap_or(&0.0));
        }

        spreadsheet_manager
            .write_named_range(
                ranges::balances::binance::RW_AMOUNTS,
                // TODO: create Vec<T> to ValueRange conversion
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
