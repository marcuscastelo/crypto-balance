use std::{collections::HashMap, rc::Rc, sync::LazyLock};

use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    config::app_config::{self, CONFIG},
    scraping::debank_scraper::{DebankBalanceScraper, ProjectTracking},
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager, ranges,
        value_range_factory::ValueRangeFactory,
    },
    Routine, RoutineFailureInfo, RoutineResult,
};

struct RelevantDebankToken {
    token_name: &'static str,
    range_name_rows: &'static str,
    range_amount_rows: &'static str,
    alternative_names: Vec<&'static str>,
}

static RELEVANT_DEBANK_TOKENS: LazyLock<Vec<RelevantDebankToken>> = LazyLock::new(|| {
    vec![
        RelevantDebankToken {
            token_name: "ETH",
            range_name_rows: ranges::AaH::RW_ETH_BALANCES_NAMES,
            range_amount_rows: ranges::AaH::RW_ETH_BALANCES_AMOUNTS,
            alternative_names: vec!["WETH"],
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

fn parse_amount(amount: &str) -> anyhow::Result<f64> {
    let amount = amount
        .replace("₁", "")
        .replace("₂", "0")
        .replace("₃", "00")
        .replace("₄", "000")
        .replace("₅", "0000")
        .replace("₆", "00000")
        .replace("₇", "000000")
        .replace("₈", "0000000")
        .replace("₉", "00000000");

    let (amount, _) = amount.split_once(" ").unwrap_or((amount.as_str(), ""));

    Ok(amount.parse()?)
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

        let mut balances = HashMap::new();
        for (chain, chain_info) in chain_infos.iter() {
            if let Some(wallet) = chain_info.wallet_info.as_ref() {
                let wallet_balances: HashMap<String, (String, f64)> = wallet
                    .tokens
                    .iter()
                    .filter_map(|token_info| {
                        let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
                            .iter()
                            .filter(|relevant_token| relevant_token.matches(&token_info.name))
                            .collect::<Vec<_>>();

                        if matching_relevant_tokens.len() > 1 {
                            log::error!(
                                "Multiple relevant tokens found for token: {}. Halt!.",
                                token_info.name
                            );
                            panic!()
                        }

                        if matching_relevant_tokens.is_empty() {
                            log::warn!("Ignoring token: {}", token_info.name);
                            return None;
                        }

                        let relevant_token = matching_relevant_tokens.first().unwrap();

                        Some((
                            relevant_token.token_name.to_owned(),
                            (
                                token_info.name.clone(),
                                parse_amount(token_info.amount.as_str()).unwrap_or_else(|e| {
                                    log::error!(
                                        "Failed to parse amount for token: {}, amount: {:?}.\n{}",
                                        token_info.name,
                                        token_info.amount,
                                        e
                                    );
                                    panic!()
                                }),
                            ),
                        ))
                    })
                    .collect();

                for (unbrella_token, (token, amount)) in wallet_balances.into_iter() {
                    let token_balances = balances
                        .entry(unbrella_token.clone())
                        .or_insert(HashMap::new());
                    token_balances.insert(format!("Wallet@{} ({})", chain, token), amount);
                }
            }

            for project in chain_info.project_info.as_slice() {
                let project_name = project.name.clone();

                for tracking in project.trackings.as_slice() {
                    match tracking {
                        ProjectTracking::YieldFarm { yield_farm } => {
                            for token in yield_farm {
                                let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
                                    .iter()
                                    .filter(|relevant_token| {
                                        let matches_pool = relevant_token.matches(&token.pool);
                                        let matches_token_name =
                                            if let Some(token_name) = token.token_name.as_ref() {
                                                relevant_token.matches(token_name)
                                            } else {
                                                false
                                            };

                                        matches_pool || matches_token_name
                                    })
                                    .collect::<Vec<_>>();

                                if matching_relevant_tokens.len() > 1 {
                                    log::error!(
                                        "Multiple relevant tokens found for token: {}. Halt!.",
                                        token.pool
                                    );
                                    panic!()
                                }

                                if matching_relevant_tokens.is_empty() {
                                    log::warn!("Ignoring token: {}", token.pool);
                                    continue;
                                }

                                let relevant_token = matching_relevant_tokens.first().unwrap();

                                let token_balances = balances
                                    .entry(relevant_token.token_name.to_owned())
                                    .or_insert(HashMap::new());

                                let amount =
                                    parse_amount(&token.balance.as_str()).unwrap_or_else(|_| {
                                        log::error!(
                                            "Failed to parse amount for token: {}, amount: {:?}",
                                            token.pool,
                                            token.balance
                                        );
                                        panic!()
                                    });

                                let name = format!("{}@{} ({})", project_name, chain, token.pool);
                                log::trace!(
                                    "{} - {}; {}",
                                    relevant_token.range_name_rows,
                                    name,
                                    amount
                                );
                                token_balances.insert(name, amount);
                            }
                        }
                        _ => {
                            log::error!("Unsupported tracking: {:?}", tracking);
                        }
                    }
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
