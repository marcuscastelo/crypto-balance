use crate::prelude::*;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use google_sheets4::api::ValueRange;
use serde_json::Value;
pub struct UpdateKrakenBalanceOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateKrakenBalanceOnSheetsRoutine {
    async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        // TODO: why is KrakenFactory on prelude
        let kraken_api = KrakenFactory::create();

        let token_names: Vec<String> = routines::sheets::SheetsGetTokenNamesRoutine.run().await;

        let balances = kraken_api
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
            .collect::<HashMap<_, _>>();

        println!("Kraken account balance: {:#?}", balances);

        // Write to the spreadsheet
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in &token_names {
            token_balances.push(balances.get(token_name).unwrap_or(&0.0));
        }

        println!(
            "Balances in order:\n{:#?}",
            token_names
                .iter()
                .zip(token_balances.clone())
                .collect::<Vec<_>>()
        );

        spreadsheet_manager
            .write_named_range(
                ranges::balances::kraken::RW_AMOUNTS,
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