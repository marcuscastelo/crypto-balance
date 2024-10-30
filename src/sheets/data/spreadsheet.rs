use google_sheets4::api::ValueRange;
use serde_json::Value;

use super::spreadsheet_manager::SpreadsheetManager;
use crate::sheets::{into::MyInto, ranges};

pub struct SpreadsheetUseCasesImpl;

impl SpreadsheetUseCasesImpl {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }
    pub async fn get_token_names_from_spreadsheet(&self) -> Vec<String> {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }

    pub async fn update_binance_balances_on_spreadsheet(&self, balances: &[f64]) {
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
