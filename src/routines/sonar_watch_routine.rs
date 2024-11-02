use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    scraping::sonar_watch_scraper::SonarWatchScraper,
    script::sonar_script::sonar_verify,
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager, ranges,
        value_range_factory::ValueRangeFactory,
    },
    Routine, RoutineFailureInfo, RoutineResult,
};

pub struct SonarWatchRoutine;

impl SonarWatchRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn update_token_if_needed(&self) {
        let needs_update = sonar_verify();
    }

    async fn get_sonar_watch_balance(&self) -> anyhow::Result<f64> {
        let sonar_response = SonarWatchScraper.scrape().await?;

        sonar_response
            .value
            .ok_or_else(|| anyhow::anyhow!("SonarWatch response did not contain a value"))
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
impl Routine for SonarWatchRoutine {
    fn name(&self) -> &'static str {
        "SonarWatch"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running SonarWatch");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("SonarWatch: 🔄  Updating token if needed");
        let updated = self.update_token_if_needed().await;

        progress.trace("SonarWatch: ☁️  Fetching SonarWatch balance");
        let balance = self
            .get_sonar_watch_balance()
            .await
            .map_err(|error| RoutineFailureInfo::new(error.to_string()))?;

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
