pub use crate::prelude::*;

pub struct SheetsGetTokenNamesRoutine;

#[async_trait::async_trait]
impl Routine<Vec<String>> for SheetsGetTokenNamesRoutine {
    async fn run(&self) -> Vec<String> {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }
}