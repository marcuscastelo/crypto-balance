use google_sheets4::api::ValueRange;
use serde_json::Value;

use super::spreadsheet_manager::SpreadsheetManager;
use crate::sheets::{into::MyInto, ranges};

pub struct SpreadsheetUseCasesImpl;

pub enum BalanceUpdateTarget {
    Binance,
    Kraken,
}

fn get_target_range(target: BalanceUpdateTarget) -> &'static str {
    match target {
        BalanceUpdateTarget::Binance => ranges::balances::binance::RW_AMOUNTS,
        BalanceUpdateTarget::Kraken => ranges::balances::kraken::RW_AMOUNTS,
    }
}

impl SpreadsheetUseCasesImpl {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }
    pub async fn get_token_names_from_spreadsheet(&self) -> Vec<String> {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content, when getting token names, can't continue without it")
            .values
            .expect("Should have values when getting token names, can't continue without them")
            .my_into()
    }

    pub async fn update_balances_on_spreadsheet(
        &self,
        target: BalanceUpdateTarget,
        balances: &[f64],
    ) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        let range = get_target_range(target);

        spreadsheet_manager
            .write_named_range(
                range,
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
