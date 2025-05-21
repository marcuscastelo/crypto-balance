use std::{collections::HashMap, sync::LazyLock};

use error_stack::{Context, Result, ResultExt};
use google_sheets4::api::ValueRange;
use tracing::instrument;

#[derive(Debug)]
pub enum DebankTokensRoutineError {
    FailedToFetchRelevantTokenAmounts,
}

impl std::fmt::Display for DebankTokensRoutineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebankTokensRoutineError: {:?}", self)
    }
}

impl Context for DebankTokensRoutineError {}

use crate::{
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
            token_name: "USDT",
            range_name_rows: ranges::AaH::RW_USDT_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_USDT_BALANCES_AMOUNTS,
            alternative_names: vec![
                "USDC",
                "DAI",
                "TUSD",
                "BUSD",
                "sUSDT",
                "USDe",
                "sUSDe",
                "USDbC",
                "USDC.e",
                "USDC(Bridged)",
                "BUSD",
                "RUSD",
            ],
        },
        RelevantDebankToken {
            token_name: "ETH",
            range_name_rows: ranges::AaH::RW_ETH_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_ETH_BALANCES_AMOUNTS,
            alternative_names: vec![
                "WETH",
                "rswETH",
                "stETH",
                "wstETH",
                "wstETH+ETH",
                "eETH",
                "weETH",
                "wrsETH",
                "ezETH",
                "cbETH",
            ],
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
            alternative_names: vec![
                "WBTC",
                "uniBTC",
                "BTCB",
                "LBTC",
                "LBTCv",
                "SolvBTC",
                "SolvBTC.BBN",
                "UBTC",
            ],
        },
        RelevantDebankToken {
            token_name: "ENA",
            range_name_rows: ranges::AaH::RW_ENA_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_ENA_BALANCES_AMOUNTS,
            alternative_names: vec!["ETHENA", "PT-sENA-24APR2025", "sENA"],
        },
        RelevantDebankToken {
            token_name: "ETHFI",
            range_name_rows: ranges::AaH::RW_ETHFI_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_ETHFI_BALANCES_AMOUNTS,
            alternative_names: vec!["sETHFI"],
        },
        RelevantDebankToken {
            token_name: "GS",
            range_name_rows: ranges::AaH::RW_GS_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_GS_BALANCES_AMOUNTS,
            alternative_names: vec!["GS (GammaSwap)", "esGS"],
        },
        RelevantDebankToken {
            token_name: "TANGO",
            range_name_rows: ranges::AaH::RW_TANGO_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_TANGO_BALANCES_AMOUNTS,
            alternative_names: vec![],
        },
        RelevantDebankToken {
            token_name: "PEAR",
            range_name_rows: ranges::AaH::RW_PEAR_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_PEAR_BALANCES_AMOUNTS,
            alternative_names: vec!["PEAR (pear.garden)"],
        },
        RelevantDebankToken {
            token_name: "INST",
            range_name_rows: ranges::AaH::RW_INST_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_INST_BALANCES_AMOUNTS,
            alternative_names: vec!["FLUID"],
        },
        RelevantDebankToken {
            token_name: "SPECTRA",
            range_name_rows: ranges::AaH::RW_SPECTRA_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_SPECTRA_BALANCES_AMOUNTS,
            alternative_names: vec![],
        },
        RelevantDebankToken {
            token_name: "HYPE",
            range_name_rows: ranges::AaH::RW_HYPE_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_HYPE_BALANCES_AMOUNTS,
            alternative_names: vec!["hbHYPE", "LHYPE", "stHYPE", "mHYPE", "WHYPE"],
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
    ) -> Result<HashMap<String, HashMap<String, f64>>, DebankTokensRoutineError> {
        let scraper = DebankBalanceScraper::new()
            .await
            .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;
        let chain_infos = scraper
            .get_chain_infos(&CONFIG.blockchain.airdrops.evm.address)
            .await
            .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;

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

    #[allow(non_snake_case)] // Specially allowed for the sake of readability of an acronym
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, f64>>,
    ) {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        for token in RELEVANT_DEBANK_TOKENS.iter() {
            let empty_hashmap = HashMap::new();
            let token_balances = balances.get(token.token_name).unwrap_or_else(|| {
                tracing::error!(
                    "Failed to get balances for token: {}. Halt!",
                    token.token_name
                );
                &empty_hashmap
            });
            // let (names, amounts): (Vec<_>, Vec<_>) =

            let mut names_amounts_tuples = token_balances
                .iter()
                .map(|(name, amount)| (name.clone(), amount.to_string()))
                .collect::<Vec<(String, String)>>();

            names_amounts_tuples.sort_by(|(name1, _), (name2, _)| name1.cmp(name2));

            let (names, amounts): (Vec<_>, Vec<_>) = names_amounts_tuples.iter().cloned().unzip();

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

    #[instrument(skip(self))]
    async fn run(&self) -> RoutineResult {
        tracing::info!("Running DebankTokensRoutine");

        tracing::info!("Debank: ☁️  Fetching AaH balances");
        let balances = self.fetch_relevant_token_amounts().await.map_err(|error| {
            RoutineFailureInfo::new(format!("Failed to fetch relevant token amounts: {}", error))
        })?;

        tracing::info!("Debank: 📝 Updating token balances (AaH)");
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances)
            .await;

        tracing::info!("Debank: ✅ Updated Debank balance on the spreadsheet");

        Ok(())
    }
}
