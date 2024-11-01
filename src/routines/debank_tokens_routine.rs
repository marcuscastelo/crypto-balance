use std::{collections::HashMap, rc::Rc, sync::LazyLock};

use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    scraping::{aah_parser::AaHParser, debank_scraper::DebankBalanceScraper},
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager, ranges,
        value_range_factory::ValueRangeFactory,
    },
    Routine, RoutineFailureInfo, RoutineResult,
};

pub struct RelevantDebankToken {
    pub token_name: &'static str,
    pub range_name_rows: &'static str,
    pub range_amount_rows: &'static str,
    pub alternative_names: Vec<&'static str>,
}

pub static RELEVANT_DEBANK_TOKENS: LazyLock<Vec<RelevantDebankToken>> = LazyLock::new(|| {
    vec![
        RelevantDebankToken {
            token_name: "ETH",
            range_name_rows: ranges::AaH::RW_ETH_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_ETH_BALANCES_AMOUNTS,
            alternative_names: vec!["WETH", "rswETH", "stETH"],
        },
        RelevantDebankToken {
            token_name: "PENDLE",
            range_name_rows: ranges::AaH::RW_PENDLE_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_PENDLE_BALANCES_AMOUNTS,
            alternative_names: vec!["vPENDLE"],
        },
        RelevantDebankToken {
            token_name: "BTC",
            range_name_rows: ranges::AaH::RW_BTC_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_BTC_BALANCES_AMOUNTS,
            alternative_names: vec!["WBTC", "uniBTC"],
        },
    ]
});

impl RelevantDebankToken {
    pub fn matches(&self, token_name: &str) -> bool {
        self.token_name == token_name
            || self
                .alternative_names
                .iter()
                .any(|alternative_name| *alternative_name == token_name)
    }
}

pub struct DebankTokensRoutine;

impl DebankTokensRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await
    }

    async fn fetch_relevant_token_amounts(
        &self,
    ) -> anyhow::Result<HashMap<String, HashMap<String, f64>>> {
        let scraper = DebankBalanceScraper::new().await?;
        let chain_infos = scraper
            .get_chain_infos(&CONFIG.blockchain.airdrops.evm.address)
            .await?;

        let mut aah_parser = AaHParser::new();

        for (chain, chain_info) in chain_infos.iter() {
            if let Some(wallet) = chain_info.wallet_info.as_ref() {
                aah_parser.parse_wallet(chain, wallet);
            }
            for project in chain_info.project_info.as_slice() {
                aah_parser.parse_project(chain, project);
            }
        }

        Ok(aah_parser.balances)
    }
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, f64>>,
    ) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        for token in RELEVANT_DEBANK_TOKENS.iter() {
            let empty_hashmap = HashMap::new();
            let token_balances = balances.get(token.token_name).unwrap_or_else(|| {
                log::error!(
                    "Failed to get balances for token: {}. Halt!",
                    token.token_name
                );
                &empty_hashmap
            });
            let (names, amounts): (Vec<_>, Vec<_>) = token_balances
                .iter()
                .map(|(name, amount)| (name.clone(), amount.to_string()))
                .unzip();

            spreadsheet_manager
                .write_named_range(
                    token.range_name_rows,
                    ValueRange::from_rows(names.as_slice()),
                )
                .await
                .expect(format!("Should write NAMES for {}", token.token_name).as_str());

            spreadsheet_manager
                .write_named_range(
                    token.range_amount_rows,
                    ValueRange::from_rows(amounts.as_slice()),
                )
                .await
                .expect(format!("Should write AMOUNTS for {}", token.token_name).as_str());
        }
    }
}

#[async_trait::async_trait]
impl Routine for DebankTokensRoutine {
    fn name(&self) -> &'static str {
        "DebankTokensRoutine"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running DebankTokensRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace(format!("Debank: ☁️  Fetching AaH balances"));
        let balances = self.fetch_relevant_token_amounts().await.map_err(|error| {
            RoutineFailureInfo::new(format!("Failed to fetch relevant token amounts: {}", error))
        })?;

        progress.trace(format!("Debank: 📝 Updating token balances (AaH)"));
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances)
            .await;

        progress.info("Debank: ✅ Updated Debank balance on the spreadsheet");
        finish_progress(&progress);

        Ok(())
    }
}
