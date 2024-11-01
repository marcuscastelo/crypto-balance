use std::collections::HashMap;

use chrono::format::parse;
use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    scraping::debank_scraper::DebankBalanceScraper,
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager, ranges,
        value_range_factory::ValueRangeFactory,
    },
    Routine, RoutineFailureInfo, RoutineResult,
};

pub struct DebankTotalUSDRoutine;

impl DebankTotalUSDRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn get_debank_balance(&self) -> anyhow::Result<f64> {
        let scraper = DebankBalanceScraper::new().await?;
        scraper
            .get_total_balance(&CONFIG.blockchain.airdrops.evm.address)
            .await
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
impl Routine for DebankTotalUSDRoutine {
    fn name(&self) -> &'static str {
        "DebankTotalUSDRoutine"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running DebankTotalUSDRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Debank: ☁️  Fetching Total Debank balance");
        let total_usd_balance = self
            .get_debank_balance()
            .await
            .map_err(|error| RoutineFailureInfo::new(error.to_string()))?;

        progress.trace(format!(
            "Debank: 📝 Updating total balance with ${:.2}",
            total_usd_balance,
        ));
        self.update_debank_balance_on_spreadsheet(total_usd_balance)
            .await;

        progress.info("Debank: ✅ Updated Debank balance on the spreadsheet");
        finish_progress(&progress);

        Ok(())
    }
}
