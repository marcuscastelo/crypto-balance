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

pub struct DebankRoutine;

impl DebankRoutine {
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

    async fn fetch_relevant_token_amounts(
        &self,
    ) -> anyhow::Result<HashMap<String, HashMap<String, f64>>> {
        let scraper = DebankBalanceScraper::new().await?;
        let chain_infos = scraper
            .get_chain_infos(&CONFIG.blockchain.airdrops.evm.address)
            .await?;

        let relevant_tokens = vec!["ETH", "PENDLE"];

        let mut balances = HashMap::new();
        for (chain, chain_info) in chain_infos.iter() {
            if let Some(wallet) = chain_info.wallet_info.as_ref() {
                let wallet_balances: HashMap<String, f64> = wallet
                    .tokens
                    .iter()
                    .filter_map(|token_info| {
                        if relevant_tokens.contains(&token_info.name.as_str()) {
                            Some((token_info.name.clone(), token_info.amount.parse().unwrap()))
                        } else {
                            None
                        }
                    })
                    .collect();

                for (token, amount) in wallet_balances.into_iter() {
                    let token_balances = balances.entry(token.clone()).or_insert(HashMap::new());
                    token_balances.insert(format!("Wallet@{} ({})", chain, token), amount);
                }
            }
        }

        Ok(balances)
    }
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, f64>>,
    ) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        let eth_iter = balances.get("ETH").unwrap().iter();
        let (names, amounts): (Vec<_>, Vec<_>) = eth_iter
            .map(|(name, amount)| (name.clone(), amount.to_string()))
            .unzip();

        spreadsheet_manager
            .write_named_range(
                ranges::AaH::RW_ETH_BALANCES_NAMES,
                ValueRange::from_rows(names.as_slice()),
            )
            .await
            .expect("Should write chain names");

        spreadsheet_manager
            .write_named_range(
                ranges::AaH::RW_ETH_BALANCES_AMOUNTS,
                ValueRange::from_rows(amounts.as_slice()),
            )
            .await
            .expect("Should write chain amounts");

        let pendle_iter = balances.get("PENDLE").unwrap().iter();
        let (names, amounts): (Vec<_>, Vec<_>) = pendle_iter
            .map(|(name, amount)| (name.clone(), amount.to_string()))
            .unzip();

        spreadsheet_manager
            .write_named_range(
                ranges::AaH::RW_PENDLE_BALANCES_NAMES,
                ValueRange::from_rows(names.as_slice()),
            )
            .await
            .expect("Should write chain names");

        spreadsheet_manager
            .write_named_range(
                ranges::AaH::RW_PENDLE_BALANCES_AMOUNTS,
                ValueRange::from_rows(amounts.as_slice()),
            )
            .await
            .expect("Should write chain amounts");
    }
}

#[async_trait::async_trait]
impl Routine for DebankRoutine {
    fn name(&self) -> &'static str {
        "DebankRoutine"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running DebankRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Debank: ☁️  Fetching Total Debank balance");
        let total_usd_balance = self
            .get_debank_balance()
            .await
            .map_err(|error| RoutineFailureInfo::new(error.to_string()))?;

        progress.trace(format!("Debank: ☁️  Fetching AaH balances"));
        let balances = self.fetch_relevant_token_amounts().await.map_err(|error| {
            RoutineFailureInfo::new(format!("Failed to fetch relevant token amounts: {}", error))
        })?;

        progress.trace(format!(
            "Debank: 📝 Updating total balance with ${:.2}",
            total_usd_balance,
        ));
        self.update_debank_balance_on_spreadsheet(total_usd_balance)
            .await;

        progress.trace(format!("Debank: 📝 Updating token balances (AaH)"));
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances)
            .await;

        progress.info("Debank: ✅ Updated Debank balance on the spreadsheet");
        finish_progress(&progress);

        Ok(())
    }
}
