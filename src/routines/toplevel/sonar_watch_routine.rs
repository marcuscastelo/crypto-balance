use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    ranges,
    sonar_watch_scraper::SonarWatchScraper,
    spreadsheet_manager::SpreadsheetManager,
    value_range_factory::ValueRangeFactory,
    Routine, RoutineFailureInfo, RoutineResult,
};

pub struct SonarWatch;

impl SonarWatch {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn get_sonar_watch_balance(&self) -> Option<f64> {
        let sonar_response = SonarWatchScraper
            .scrape()
            .await
            .expect("Should get SonarWatch total balance");

        sonar_response.value
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
    fn name(&self) -> &'static str {
        "SonarWatch"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running SonarWatch");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("SonarWatch: ☁️  Fetching SonarWatch balance");
        let balance = self.get_sonar_watch_balance().await.ok_or_else(|| {
            RoutineFailureInfo::new("Unable to get SonarWatch balance".to_owned())
        })?;

        progress.trace(format!(
            "SonarWatch: 📝 Updating balance with ${:.2}",
            balance,
        ));
        self.update_sonar_watch_balance_on_spreadsheet(balance)
            .await;

        progress.info("SonarWatch: ✅ Updated SonarWatch balance on the spreadsheet");
        finish_progress(&progress);

        Ok(())
    }
}
