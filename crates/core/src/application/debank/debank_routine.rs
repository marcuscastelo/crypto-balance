use core::fmt;
use error_stack::ResultExt;
use std::sync::Arc;
use std::{collections::HashMap, sync::LazyLock, vec};
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::adapters::config::blockchain_config::MultiEvmBlockchainConfig;
use crate::adapters::debank::aah_parser::{AaHParser, TokenBalance};
use crate::adapters::debank::balance::format_balance;
use crate::adapters::sheets::spreadsheet_manager::{SpreadsheetManager, SpreadsheetManagerError};
use crate::adapters::sheets::spreadsheet_write::SpreadsheetWrite;
use crate::domain::debank::{Chain, DebankResponse};
use crate::domain::routine::{Routine, RoutineError};
use crate::domain::sheets::ranges;

// Minimum USD value for positions to be included in the spreadsheet
const MIN_USD_VALUE: f64 = 1.0;

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
            token_name: "USD".into(),
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
                "hbUSDT",
                "WHLP",
                "USDHL",
                "THBILL",
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
                "weETHs",
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
            alternative_names: vec!["ETHENA", "PT-sENA-24APR2025", "sENA", "ENA"],
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
    config: MultiEvmBlockchainConfig,
    spreadsheet_manager: Arc<SpreadsheetManager>,
}

impl fmt::Debug for DebankRoutine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebankRoutine").finish()
    }
}

impl DebankRoutine {
    #[instrument]
    pub fn new(
        config: MultiEvmBlockchainConfig,
        spreadsheet_manager: Arc<SpreadsheetManager>,
    ) -> Self {
        Self {
            config,
            spreadsheet_manager,
        }
    }

    #[instrument(skip(self), name = "DebankRoutine::load_debank_data")]
    async fn load_debank_data(
        &self,
        wallet_address: &str,
    ) -> error_stack::Result<DebankResponse, RoutineError> {
        tracing::debug!(
            wallet_address = wallet_address,
            "Loading Debank data via API"
        );

        // Create API client
        let api_client = crate::adapters::debank::api_client::DebankApiClient::new(
            "http://localhost:8000".to_string(),
        );

        // Create scrape request
        let scrape_request = crate::adapters::debank::api_client::ScrapeRequest {
            wallet_address: wallet_address.to_string(),
            chain: None, // No chain filter by default
            save_html: false,
            save_screenshot: false,
            headless: true,
        };

        // Scrape wallet data via API
        let debank_response = api_client
            .scrape_wallet(scrape_request)
            .await
            .change_context(RoutineError::routine_failure(
                "Failed to scrape wallet data via API".to_string(),
            ))?;

        tracing::debug!(
            chains_loaded = debank_response.chains.len(),
            total_balance = ?debank_response.metadata.as_ref().map(|m| &m.wallet_address),
            total_usd_value = %debank_response.total_usd_value,
            "Successfully loaded Debank data via API"
        );

        Ok(debank_response)
    }

    #[instrument(skip(self, chains), name = "DebankRoutine::parse_debank_profile")]
    async fn parse_debank_profile(
        &self,
        chains: &[Chain],
    ) -> error_stack::Result<
        (HashMap<String, HashMap<String, TokenBalance>>, Vec<String>),
        DebankTokensRoutineError,
    > {
        let mut aah_parser = AaHParser::new();
        let chain_order: Vec<String> = chains.iter().map(|chain| chain.name.clone()).collect();

        for chain in chains.iter() {
            event!(Level::TRACE, chain = chain.name, "Parsing chain");
            if let Some(wallet) = chain.wallet_info.as_ref() {
                event!(Level::TRACE, wallet = ?wallet, "Wallet detected, parsing");
                aah_parser
                    .parse_wallet(&chain.name, wallet)
                    .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)?;
            }
            for project in chain.project_info.as_slice() {
                event!(
                    Level::TRACE,
                    project = project.name,
                    "Project detected, parsing"
                );
                aah_parser
                    .parse_project(&chain.name, project)
                    .change_context(DebankTokensRoutineError::FailedToFetchRelevantTokenAmounts)
                    .attach_printable_lazy(|| {
                        format!(
                            "Failed to parse project: {} on chain: {}",
                            project.name, chain.name
                        )
                    })?;
            }
        }

        Ok((aah_parser.balances, chain_order))
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

    #[instrument(
        skip(self, balances, chain_order),
        name = "DebankRoutine::update_debank_eth_AaH_balances_on_spreadsheet"
    )]
    #[allow(non_snake_case)] // Specially allowed for the sake of readability of an acronym
    async fn update_debank_eth_AaH_balances_on_spreadsheet(
        &self,
        balances: HashMap<String, HashMap<String, TokenBalance>>,
        chain_order: Vec<String>,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        futures::future::join_all(
            RELEVANT_DEBANK_TOKENS
                .iter()
                .map(|token| self.update_balances_for_token(token, &balances, &chain_order))
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
        balances: &HashMap<String, HashMap<String, TokenBalance>>,
        chain_order: &Vec<String>,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let empty_hashmap = HashMap::new();
        let token_balances = balances.get(token.token_name).unwrap_or_else(|| {
            tracing::warn!(name = token.token_name, token = ?token, "Token not found in balances");
            &empty_hashmap
        });

        let mut names_amounts_tuples = token_balances
            .iter()
            .filter_map(|(name, token_balance)| {
                // Filter out positions with USD value below $1.00, but only if USD value is Some
                let should_include = match token_balance.usd_value {
                    Some(usd_val) => (usd_val * usd_val) >= MIN_USD_VALUE,
                    None => true, // Always include when USD value is None
                };

                if should_include {
                    Some((name.clone(), token_balance.amount.to_string()))
                } else {
                    tracing::debug!(
                        token = token.token_name,
                        position = name,
                        usd_value = ?token_balance.usd_value,
                        amount = token_balance.amount,
                        "Filtered out position with USD value below ${:.2}",
                        MIN_USD_VALUE
                    );
                    None
                }
            })
            .collect::<Vec<(String, String)>>();

        tracing::debug!(
            token = token.token_name,
            total_positions = token_balances.len(),
            filtered_positions = names_amounts_tuples.len(),
            "Applied $1.00 minimum filter"
        );

        // Create a map of chain names to their order for efficient lookup
        let chain_order_map: HashMap<&String, usize> = chain_order
            .iter()
            .enumerate()
            .map(|(i, chain)| (chain, i))
            .collect();

        // Custom sort: first by chain order (from JSON), then alphabetically within same chain
        names_amounts_tuples.sort_by(|(name1, _), (name2, _)| {
            // Extract chain from the location name (format is "chain - <custody> (token)")
            let chain1 = name1.split(" - ").next().unwrap_or(name1);
            let chain2 = name2.split(" - ").next().unwrap_or(name2);

            let order1 = chain_order_map
                .get(&chain1.to_string())
                .unwrap_or(&usize::MAX);
            let order2 = chain_order_map
                .get(&chain2.to_string())
                .unwrap_or(&usize::MAX);

            tracing::trace!(
                "Comparing '{}' (chain: '{}', order: {:?}) vs '{}' (chain: '{}', order: {:?})",
                name1,
                chain1,
                order1,
                name2,
                chain2,
                order2
            );

            // First compare by chain order
            match order1.cmp(order2) {
                std::cmp::Ordering::Equal => {
                    // If same chain (or both not found), sort alphabetically
                    name1.cmp(name2)
                }
                other => other,
            }
        });

        let (names, amounts): (Vec<_>, Vec<_>) = names_amounts_tuples.iter().cloned().unzip();

        tracing::debug!(
            token_name = token.token_name,
            entries_count = names.len(),
            entries = ?names,
            "Final sorted order for token"
        );

        self.spreadsheet_manager
            .write_named_two_columns(
                token.range_balance_two_cols,
                names.as_slice(),
                amounts.as_slice(),
            )
            .await?;

        Ok(())
    }

    async fn process_wallet(
        &self,
        user_id: String,
    ) -> error_stack::Result<
        (
            String,
            HashMap<String, HashMap<String, TokenBalance>>,
            Vec<String>,
            f64,
        ),
        RoutineError,
    > {
        // Load chains from API
        let debank_response = self.load_debank_data(user_id.as_str()).await?;
        tracing::debug!(
            chains_count = debank_response.chains.len(),
            total_usd_value = debank_response.total_usd_value,
            "Chains loaded from API"
        );

        let (balances, chain_order) = self
            .parse_debank_profile(debank_response.chains.as_ref())
            .await
            .change_context(RoutineError::routine_failure(format!(
                "Failed to parse Debank profile: {}",
                user_id
            )))?;
        tracing::debug!(
            balances = ?balances,
            "Balances processed"
        );

        // Use total USD value from API instead of manually calculating
        let total_balance = format_balance(&debank_response.total_usd_value).map_err(|e| {
            RoutineError::routine_failure(format!(
                "Failed to parse total USD value '{}' from API: {}",
                debank_response.total_usd_value, e
            ))
        })?;

        Ok((user_id, balances, chain_order, total_balance))
    }

    #[instrument(skip(self), name = "DebankRoutine::main_routine")]
    async fn main_routine(&self) -> error_stack::Result<(), RoutineError> {
        let (balances, chain_order, total_balance) = {
            let mut futures = vec![];
            for address in self.config.addresses.iter() {
                let address = address.trim();
                if address.is_empty() {
                    tracing::warn!("Skipping empty wallet address in configuration");
                    continue;
                }
                futures.push(self.process_wallet(address.to_owned()));
            }

            let results = futures::future::join_all(futures).await;
            if results.is_empty() {}

            let mut combined_balances: HashMap<String, HashMap<String, TokenBalance>> =
                HashMap::new();
            let mut combined_chain_order: Vec<String> = vec![];
            let mut total_balance = 0.0;

            for result in results {
                match result {
                    Ok((address, result_balances, result_chain_order, result_total_balance)) => {
                        // Merge balances
                        for (token_name, token_balances) in result_balances {
                            let combined_entry = combined_balances
                                .entry(token_name)
                                .or_insert_with(HashMap::new);
                            for (token_location, balance) in token_balances {
                                let location_with_address = format!(
                                    "{} ({})",
                                    token_location,
                                    address.get(0..6).unwrap_or("<unknown address>")
                                );
                                combined_entry.insert(location_with_address, balance);
                            }
                        }

                        // Merge chain order, preserving order and avoiding duplicates
                        for chain in result_chain_order {
                            if !combined_chain_order.contains(&chain) {
                                combined_chain_order.push(chain);
                            }
                        }

                        total_balance += result_total_balance;
                    }
                    Err(err) => {
                        tracing::error!(error = ?err, "Error processing wallet, skipping");
                        return Err(err);
                    }
                }
            }

            (combined_balances, combined_chain_order, total_balance)
        };

        tracing::debug!(
            total_balance = total_balance,
            "Using total balance from API"
        );

        tracing::trace!("Updating TOTAL balance on the spreadsheet");
        self.update_debank_balance_on_spreadsheet(total_balance)
            .await
            .change_context(RoutineError::routine_failure(format!(
                "Failed to update Debank balance on the spreadsheet"
            )))?;

        tracing::trace!("Updating AaH balances on the spreadsheet");
        self.update_debank_eth_AaH_balances_on_spreadsheet(balances, chain_order)
            .await
            .change_context(RoutineError::routine_failure(format!(
                "Failed to update Debank AaH balances on the spreadsheet"
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
