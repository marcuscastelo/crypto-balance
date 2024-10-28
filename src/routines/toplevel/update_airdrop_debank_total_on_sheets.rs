use google_sheets4::api::ValueRange;

use crate::{
    config::app_config::{self, CONFIG},
    ranges, DebankScraper, Routine, SpreadsheetManager, ValueRangeFactory,
};

pub struct UpdateAirdropDebankTotalOnSheetsRoutine;

#[async_trait::async_trait]
impl Routine for UpdateAirdropDebankTotalOnSheetsRoutine {
    async fn run(&self) {
        log::info!("Running UpdateAirdropDebankTotalOnSheetsRoutine");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = DebankScraper
            .get_total_balance(&CONFIG.blockchain.airdrops.evm.address)
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
