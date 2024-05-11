use crate::prelude::*;
use google_sheets4::api::ValueRange;

pub struct UpdateAirdropDebankTotalOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateAirdropDebankTotalOnSheetsRoutine {
    async fn run(&self) {
        info!("Running UpdateAirdropDebankTotalOnSheetsRoutine");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = DebankScraper
            .get_total_balance(&CONFIG.blockchain.evm.address)
            .await
            .expect("Should get Debank total balance");

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_DEBANK_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write Debank total to the spreadsheet");
    }
}
