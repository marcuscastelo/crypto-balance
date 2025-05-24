use core::fmt;
use error_stack::{Context, Result, ResultExt};
use std::{collections::HashMap, sync::LazyLock, vec};
use tracing::{event, instrument, Level};

use crate::{
    config::app_config::CONFIG,
    scraping::{
        aah_parser::AaHParser,
        debank_scraper::{ChainInfo, DebankBalanceScraper},
    },
    sheets::{
        data::spreadsheet_manager::{SpreadsheetManager, SpreadsheetManagerError},
        ranges,
    },
    Routine,
};

use super::routine::RoutineError;

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

#[derive(Debug)]
pub struct RelevantDebankToken {
    pub token_name: &'static str,
    pub range_balance_two_cols: &'static str,
    pub alternative_names: Vec<&'static str>,
}

pub static RELEVANT_DEBANK_TOKENS: LazyLock<Vec<RelevantDebankToken>> = LazyLock::new(|| {
    vec![
        RelevantDebankToken {
            token_name: "USDT",
            range_balance_two_cols: ranges::AaH::RW_USDT_BALANCES_NAMES,
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
            range_balance_two_cols: ranges::AaH::RW_ETH_BALANCES_NAMES,
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
            range_balance_two_cols: ranges::AaH::RW_PENDLE_BALANCES_NAMES,
            alternative_names: vec!["vPENDLE"],
        },
        RelevantDebankToken {
            token_name: "BTC",
            range_balance_two_cols: ranges::AaH::RW_BTC_BALANCES_NAMES,
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
            range_balance_two_cols: ranges::AaH::RW_ENA_BALANCES_NAMES,
            alternative_names: vec!["ETHENA", "PT-sENA-24APR2025", "sENA"],
        },
        RelevantDebankToken {
            token_name: "ETHFI",
            range_balance_two_cols: ranges::AaH::RW_ETHFI_BALANCES_NAMES,
            alternative_names: vec!["sETHFI"],
        },
        RelevantDebankToken {
            token_name: "GS",
            range_balance_two_cols: ranges::AaH::RW_GS_BALANCES_NAMES,
            alternative_names: vec!["GS (GammaSwap)", "esGS"],
        },
        RelevantDebankToken {
            token_name: "TANGO",
            range_balance_two_cols: ranges::AaH::RW_TANGO_BALANCES_NAMES,
            alternative_names: vec![],
        },
        RelevantDebankToken {
            token_name: "PEAR",
            range_balance_two_cols: ranges::AaH::RW_PEAR_BALANCES_NAMES,
            alternative_names: vec!["PEAR (pear.garden)"],
        },
        RelevantDebankToken {
            token_name: "INST",
            range_balance_two_cols: ranges::AaH::RW_INST_BALANCES_NAMES,
            alternative_names: vec!["FLUID"],
        },
        RelevantDebankToken {
            token_name: "SPECTRA",
            range_balance_two_cols: ranges::AaH::RW_SPECTRA_BALANCES_NAMES,
            alternative_names: vec![],
        },
        RelevantDebankToken {
            token_name: "HYPE",
            range_balance_two_cols: ranges::AaH::RW_HYPE_BALANCES_NAMES,
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

pub struct DebankRoutine<'s> {
    spreadsheet_manager: &'s SpreadsheetManager,
}

impl<'s> fmt::Debug for DebankRoutine<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebankRoutine").finish()
    }
}

impl<'s> DebankRoutine<'s> {
    #[instrument]
    pub fn new(spreadsheet_manager: &'s SpreadsheetManager) -> Self {
        Self {
            spreadsheet_manager,
        }
    }

    #[instrument(skip(self), name = "DebankRoutine::create_scraper")]
    async fn create_scraper(&self) -> DebankBalanceScraper {
        let scraper = DebankBalanceScraper::new()
            .await
            .expect("Should create DebankBalanceScraper");

        scraper
    }

    #[instrument(skip(self), name = "DebankRoutine::fetch_relevant_token_amounts", fields(user_id = CONFIG.blockchain.airdrops.evm.address))]
    async fn fetch_relevant_token_amounts(
        &self,
    ) -> Result<HashMap<String, HashMap<String, f64>>, DebankTokensRoutineError> {
        let scraper = DebankBalanceScraper::new()
            .await
            .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;
        let chain_infos = scraper
            .explore_debank_profile(&CONFIG.blockchain.airdrops.evm.address)
            .await
            .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;

        return self.parse_debank_profile(chain_infos).await;
    }

    #[instrument(skip(self, chain_infos), name = "DebankRoutine::parse_debank_profile", fields(user_id = CONFIG.blockchain.airdrops.evm.address))]
    async fn parse_debank_profile(
        &self,
        chain_infos: HashMap<String, ChainInfo>,
    ) -> Result<HashMap<String, HashMap<String, f64>>, DebankTokensRoutineError> {
        let mut aah_parser = AaHParser::new();

        for (chain, chain_info) in chain_infos.iter() {
            event!(Level::TRACE, chain = chain, "Parsing chain");
            if let Some(wallet) = chain_info.wallet_info.as_ref() {
                event!(Level::TRACE, wallet = ?wallet, "Wallet detected, parsing");
                aah_parser.parse_wallet(chain, wallet);
            }
            for project in chain_info.project_info.as_slice() {
                event!(
                    Level::TRACE,
                    project = project.name,
                    "Project detected, parsing"
                );
                aah_parser.parse_project(chain, project);
            }
        }

        Ok(aah_parser.balances)
    }

    #[instrument]
    async fn update_debank_balance_on_spreadsheet(
        &self,
        balance: f64,
    ) -> Result<(), SpreadsheetManagerError> {
        self.spreadsheet_manager
            .write_named_cell(ranges::airdrops::RW_DEBANK_TOTAL_USD, &balance.to_string())
            .await?;

        Ok(())
    }

    #[instrument]
    #[allow(non_snake_case)] // Specially allowed for the sake of readability of an acronym
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, f64>>,
    ) -> Result<(), SpreadsheetManagerError> {
        futures::future::join_all(
            RELEVANT_DEBANK_TOKENS
                .iter()
                .map(|token| self.update_balances_for_token(token, &balances))
                .collect::<Vec<_>>(),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    #[instrument]
    async fn update_balances_for_token(
        &self,
        token: &RelevantDebankToken,
        balances: &HashMap<String, HashMap<String, f64>>,
    ) -> Result<(), SpreadsheetManagerError> {
        let empty_hashmap = HashMap::new();
        let token_balances = balances.get(token.token_name).unwrap_or_else(|| {
            tracing::warn!(name = token.token_name, token = ?token, "Token not found in balances");
            &empty_hashmap
        });

        let mut names_amounts_tuples = token_balances
            .iter()
            .map(|(name, amount)| (name.clone(), amount.to_string()))
            .collect::<Vec<(String, String)>>();

        names_amounts_tuples.sort_by(|(name1, _), (name2, _)| name1.cmp(name2));

        let (names, amounts): (Vec<_>, Vec<_>) = names_amounts_tuples.iter().cloned().unzip();

        self.spreadsheet_manager
            .write_named_two_columns(
                token.range_balance_two_cols,
                names.as_slice(),
                amounts.as_slice(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self), name = "DebankRoutine::main_routine")]
    async fn main_routine(&self) -> error_stack::Result<(), RoutineError> {
        let scraper = self.create_scraper().await;
        let user_id = CONFIG.blockchain.airdrops.evm.address.as_ref();

        tracing::debug!(user_id = user_id, "Accessing Debank profile");
        scraper
            .access_profile(user_id)
            .await
            .change_context(RoutineError::RoutineFailure(format!(
                "Failed to access Debank profile: {}",
                user_id
            )))?;
        tracing::debug!(user_id = user_id, "Accessed Debank profile");

        let total_balance =
            scraper
                .get_total_balance()
                .await
                .change_context(RoutineError::RoutineFailure(format!(
                    "Failed to get total balance for user: {}",
                    user_id
                )))?;
        tracing::debug!(total_balance = total_balance, "Total balance processed");

        let scraped_chains = scraper
            .explore_debank_profile(user_id)
            .await
            .change_context(RoutineError::RoutineFailure(format!(
                "Failed to explore Debank profile: {}",
                user_id
            )))?;
        tracing::debug!(scraped_chains = ?scraped_chains, "Scraped chains processed");

        let balances = self
            .parse_debank_profile(scraped_chains)
            .await
            .change_context(RoutineError::RoutineFailure(format!(
                "Failed to parse Debank profile: {}",
                user_id
            )))?;

        tracing::debug!(
            balances = ?balances,
            "Balances processed"
        );

        tracing::trace!("Updating TOTAL balance on the spreadsheet");
        self.update_debank_balance_on_spreadsheet(total_balance)
            .await
            .change_context(RoutineError::RoutineFailure(format!(
                "Failed to update Debank balance on the spreadsheet for wallet: {}",
                user_id
            )))?;

        tracing::trace!("Updating AaH balances on the spreadsheet");
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances)
            .await
            .change_context(RoutineError::RoutineFailure(format!(
                "Failed to update Debank AaH balances on the spreadsheet for wallet: {}",
                user_id
            )))?;

        tracing::info!("Debank: ✅ Updated Debank balance on the spreadsheet");
        Ok(())
    }

    #[instrument(skip(self), name = "DebankRoutine::prefetch_named_ranges")]
    async fn prefetch_named_ranges(&self) -> error_stack::Result<(), RoutineError> {
        let _ = self.spreadsheet_manager.named_range_map().await.map_err(|err|
            tracing::warn!(error = ?err, "Prefetching named ranges failed, falling back to default")
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl<'s> Routine for DebankRoutine<'s> {
    fn name(&self) -> &'static str {
        "DebankRoutine"
    }

    #[instrument(skip(self), name = "DebankRoutine::run")]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("Running DebankRoutine");
        futures::future::try_join(self.prefetch_named_ranges(), self.main_routine()).await?;
        Ok(())
    }
}
