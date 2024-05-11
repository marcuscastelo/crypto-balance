use crate::prelude::*;
use google_sheets4::api::ValueRange;

pub struct UpdateAirdropStepSVMTotalOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateAirdropStepSVMTotalOnSheetsRoutine {
    async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = StepSVMScraper
            .get_total_balance(&CONFIG.blockchain.solana.address)
            .await
            .unwrap();

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_STEP_SVM_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write StepSVM total to the spreadsheet");
    }
}