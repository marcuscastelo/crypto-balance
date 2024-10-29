use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    ranges,
    sonar_watch::SonarWatchScraper,
    spreadsheet_manager::SpreadsheetManager,
    value_range_factory::ValueRangeFactory,
    Routine,
};

pub struct SonarWatch;

impl SonarWatch {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn get_sonar_watch_balance(&self) -> f64 {
        SonarWatchScraper
            .get_total_balance(&CONFIG.blockchain.airdrops.solana.address)
            .await
            .expect("Should get SonarWatch total balance")
    }

    async fn update_sonar_watch_balance_on_spreadsheet(&self, balance: f64) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_SONAR_WATCH_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write SonarWatch total to the spreadsheet");
    }
}

#[async_trait::async_trait]
impl Routine for SonarWatch {
    async fn run(&self) {
        log::info!("Running UpdateAirdropSonarWatchTotalOnSheetsRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("SonarWatch: ☁️  Fetching SonarWatch balance");
        let balance = self.get_sonar_watch_balance().await;

        progress.trace(format!(
            "SonarWatch: 📝 Updating balance with ${:.2}",
            balance,
        ));
        self.update_sonar_watch_balance_on_spreadsheet(balance)
            .await;

        progress.info("SonarWatch: ✅ Updated SonarWatch balance on the spreadsheet");
        finish_progress(&progress);
    }
}
