use google_sheets4::api::ValueRange;
use struct_name::StructName;
use struct_name_derive::StructName;
use tracing::instrument;

use crate::{
    config::app_config::{self, CONFIG},
    scraping::debank_scraper::DebankBalanceScraper,
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager, ranges,
        value_range_factory::ValueRangeFactory,
    },
    Routine, RoutineFailureInfo, RoutineResult,
};

#[derive(Debug, StructName)]
pub struct DebankTotalUSDRoutine;

impl DebankTotalUSDRoutine {
    #[instrument]
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    #[instrument]
    async fn get_debank_balance(&self) -> anyhow::Result<f64> {
        let scraper = DebankBalanceScraper::new()
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
        scraper
            .get_total_balance(&CONFIG.blockchain.airdrops.evm.address)
            .await
            .map_err(|error| anyhow::anyhow!(error))
    }

    #[instrument]
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
        self.struct_name()
    }

    #[instrument(skip(self))]
    async fn run(&self) -> RoutineResult {
        tracing::info!("Running DebankTotalUSDRoutine");

        tracing::info!("Debank: ☁️  Fetching Total Debank balance");
        let total_usd_balance = self
            .get_debank_balance()
            .await
            .map_err(|error| RoutineFailureInfo::new(error.to_string()))?;

        tracing::info!(
            "Debank: 📝 Updating total balance with ${:.2}",
            total_usd_balance
        );
        self.update_debank_balance_on_spreadsheet(total_usd_balance)
            .await;

        tracing::info!("Debank: ✅ Updated Debank balance on the spreadsheet");

        Ok(())
    }
}
