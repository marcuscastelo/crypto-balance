use core::fmt;
use error_stack::ResultExt;
use std::sync::Arc;
use std::{collections::HashMap, sync::LazyLock, vec};
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::domain::debank::{Chain, DebankResponse};
use crate::domain::routine::{Routine, RoutineError};
use crate::domain::sheets::ranges;
use crate::infrastructure::config::blockchain_config::EvmBlockchainConfig;
use crate::infrastructure::debank::aah_parser::AaHParser;
use crate::infrastructure::sheets::spreadsheet_manager::{
    SpreadsheetManager, SpreadsheetManagerError,
};
use crate::infrastructure::sheets::spreadsheet_write::SpreadsheetWrite;

#[derive(Error, Debug)]
pub enum DebankTokensRoutineError {
    #[error("Failed to fetch relevant token amounts from Debank")]
    FailedToFetchRelevantTokenAmounts,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RelevantDebankToken {
    pub token_name: &'static str,
    pub range_balance_two_cols: &'static str,
    pub alternative_names: Vec<&'static str>,
}

impl RelevantDebankToken {
    pub fn matches(&self, token_name: &str) -> TokenMatch {
        let names = vec![self.token_name]
            .into_iter()
            .chain(self.alternative_names.iter().cloned())
            .collect::<Vec<_>>();

        let exact_match = || names.iter().any(|name| *name == token_name);
        let similar_match = || {
            names
                .iter()
                .any(|name| token_name.to_lowercase().contains(&name.to_lowercase()))
        };

        if exact_match() {
            TokenMatch::ExactMatch
        } else if similar_match() {
            TokenMatch::SimilarMatch(format!(
                "Token '{}' is similar to '{}', but didn't match any of the known names: [{:}]",
                token_name,
                self.token_name,
                names.join(", ")
            ))
        } else {
            TokenMatch::NoMatch
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenMatch {
    ExactMatch,
    SimilarMatch(String),
    NoMatch,
}

pub static RELEVANT_DEBANK_TOKENS: LazyLock<Vec<RelevantDebankToken>> = LazyLock::new(|| {
    vec![
        RelevantDebankToken {
            token_name: "USD",
            range_balance_two_cols: ranges::AaH::RW_USDT_BALANCES_NAMES,
            alternative_names: vec![
                "USDT",
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
                "USDX(Stables Labs)",
                "atUSD",
                "GHO",
                "lvlUSD",
                "USD₮0",
                "rUSD",
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
                "UETH",
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
                "cbBTC",
            ],
        },
        RelevantDebankToken {
            token_name: "ENA",
            range_balance_two_cols: ranges::AaH::RW_ENA_BALANCES_NAMES,
            alternative_names: vec!["ETHENA", "PT-sENA-24APR2025", "sENA"],
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

pub struct DebankRoutine {
    config: EvmBlockchainConfig,
    spreadsheet_manager: Arc<SpreadsheetManager>,
}

impl fmt::Debug for DebankRoutine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebankRoutine").finish()
    }
}

impl DebankRoutine {
    #[instrument]
    pub fn new(config: EvmBlockchainConfig, spreadsheet_manager: Arc<SpreadsheetManager>) -> Self {
        Self {
            config,
            spreadsheet_manager,
        }
    }

    #[instrument(skip(self), name = "DebankRoutine::load_debank_json")]
    async fn load_debank_json(&self) -> error_stack::Result<HashMap<String, Chain>, RoutineError> {
        let json_path = "debank_output.json";

        tracing::debug!(path = json_path, "Loading Debank JSON file");

        let json_content = std::fs::read_to_string(json_path).change_context(
            RoutineError::routine_failure(format!("Failed to read JSON file: {}", json_path)),
        )?;

        let debank_response: DebankResponse = serde_json::from_str(&json_content).change_context(
            RoutineError::routine_failure(format!("Failed to parse JSON file: {}", json_path)),
        )?;

        // Convert Vec<Chain> to HashMap<String, Chain> as expected by existing logic
        let mut chain_map = HashMap::new();
        for chain in debank_response.chains {
            chain_map.insert(chain.name.clone(), chain);
        }

        tracing::debug!(
            chains_loaded = chain_map.len(),
            total_balance = ?debank_response.metadata.map(|m| m.wallet_address),
            "Successfully loaded Debank data from JSON"
        );

        Ok(chain_map)
    }

    #[instrument(skip(self, chain_infos), name = "DebankRoutine::parse_debank_profile", fields(user_id = self.config.address))]
    async fn parse_debank_profile(
        &self,
        chain_infos: HashMap<String, Chain>,
    ) -> error_stack::Result<HashMap<String, HashMap<String, f64>>, DebankTokensRoutineError> {
        let mut aah_parser = AaHParser::new();

        for (chain, chain_info) in chain_infos.iter() {
            event!(Level::TRACE, chain = chain, "Parsing chain");
            if let Some(wallet) = chain_info.wallet_info.as_ref() {
                event!(Level::TRACE, wallet = ?wallet, "Wallet detected, parsing");
                aah_parser
                    .parse_wallet(chain, wallet)
                    .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;
            }
            for project in chain_info.project_info.as_slice() {
                event!(
                    Level::TRACE,
                    project = project.name,
                    "Project detected, parsing"
                );
                aah_parser
                    .parse_project(chain, project)
                    .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)
                    .attach_printable_lazy(|| {
                        format!(
                            "Failed to parse project: {} on chain: {}",
                            project.name, chain
                        )
                    })?;
            }
        }

        Ok(aah_parser.balances)
    }

    #[instrument]
    async fn update_debank_balance_on_spreadsheet(
        &self,
        balance: f64,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        self.spreadsheet_manager
            .write_named_cell(ranges::airdrops::RW_DEBANK_TOTAL_USD, &balance.to_string())
            .await?;

        Ok(())
    }

    #[instrument(skip(self, balances), name = "DebankRoutine::update_debank_eth_AaH_balances_on_spreadsheet", fields(user_id = self.config.address))]
    #[allow(non_snake_case)] // Specially allowed for the sake of readability of an acronym
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, f64>>,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
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
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
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
        let user_id = self.config.address.as_ref();

        tracing::debug!(user_id = user_id, "Processing Debank data from JSON");

        // Load chains from JSON file
        let scraped_chains = self.load_debank_json().await?;
        tracing::debug!(
            chains_count = scraped_chains.len(),
            "Chains loaded from JSON"
        );

        let balances = self
            .parse_debank_profile(scraped_chains)
            .await
            .change_context(RoutineError::routine_failure(format!(
                "Failed to parse Debank profile: {}",
                user_id
            )))?;

        tracing::debug!(
            balances = ?balances,
            "Balances processed"
        );

        // Calculate total balance from all chains (for now, we'll calculate it from the parsed data)
        let total_balance = balances
            .values()
            .flat_map(|token_map| token_map.values())
            .sum::<f64>();

        tracing::debug!(total_balance = total_balance, "Calculated total balance");

        tracing::trace!("Updating TOTAL balance on the spreadsheet");
        self.update_debank_balance_on_spreadsheet(total_balance)
            .await
            .change_context(RoutineError::routine_failure(format!(
                "Failed to update Debank balance on the spreadsheet for wallet: {}",
                user_id
            )))?;

        tracing::trace!("Updating AaH balances on the spreadsheet");
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances)
            .await
            .change_context(RoutineError::routine_failure(format!(
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
impl Routine for DebankRoutine {
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
