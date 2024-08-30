use crate::prelude::*;
use google_sheets4::api::ValueRange;

pub struct UpdateSolanaTotalOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateSolanaTotalOnSheetsRoutine {
    async fn run(&self) {
        info!("Running UpdateAirdropStepSVMTotalOnSheetsRoutine");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = StepSVMScraper
            // let balance = SonarWatchScraper
            .get_total_balance(&CONFIG.blockchain.airdrops.solana.address)
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
