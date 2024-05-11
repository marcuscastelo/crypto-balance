pub use crate::prelude::*;

pub struct SheetsGetTokenIDsRoutine;

#[async_trait::async_trait]
impl Routine<Vec<String>> for SheetsGetTokenIDsRoutine {
    async fn run(&self) -> Vec<String> {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_IDS)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }
}
