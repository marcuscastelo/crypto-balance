use std::time::Duration;

use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    ranges,
    spreadsheet_manager::SpreadsheetManager,
    value_range_factory::ValueRangeFactory,
    DebankScraper, Routine,
};

pub struct DebankRoutine;

impl DebankRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn get_debank_balance(&self) -> f64 {
        DebankScraper
            .get_total_balance(&CONFIG.blockchain.airdrops.evm.address)
            .await
            .expect("Should get Debank total balance")
    }

    async fn update_debank_balance_on_spreadsheet(&self, balance: f64) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_DEBANK_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write Debank total to the spreadsheet");
    }
}

#[async_trait::async_trait]
impl Routine for DebankRoutine {
    async fn run(&self) {
        log::info!("Running UpdateAirdropDebankTotalOnSheetsRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Debank: ☁️  Fetching Debank balance");
        let balance = self.get_debank_balance().await;

        progress.trace(format!("Debank: 📝 Updating balance with ${:.2}", balance,));
        self.update_debank_balance_on_spreadsheet(balance).await;

        progress.info("Debank: ✅ Updated Debank balance on the spreadsheet");
        finish_progress(&progress);
    }
}
