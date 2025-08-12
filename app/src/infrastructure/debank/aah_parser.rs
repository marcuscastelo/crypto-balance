use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    vec,
};

use crate::{
    application::debank::debank_routine::{
        RelevantDebankToken, TokenMatch, RELEVANT_DEBANK_TOKENS,
    },
    domain::debank::SimpleTokenInfo,
    infrastructure::debank::balance::format_balance,
};

use crate::domain::debank::{
    ChainWallet, LendingTokenInfo, Project, ProjectTracking, StakeTokenInfo, TokenInfo,
};
use error_stack::{report, ResultExt};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub amount: f64,
    pub usd_value: Option<f64>,
}

#[derive(Debug)]
pub struct AaHParser {
    pub balances: HashMap<String, HashMap<String, TokenBalance>>,
}

#[derive(Error, Debug)]
pub enum AaHParserError {
    #[error("Parsing error: {0}")]
    Parse(ParseError),

    #[error("Token match error: {0}")]
    TokenMatch(TokenMatchError),

    #[error("Unknown tracking type: {0}, cannot parse token: {1:?}")]
    UnknownTrackingType(String, TokenInfo),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to parse amount: {0}")]
    Amount(String),
}

#[derive(Error, Debug)]
pub enum TokenMatchError {
    #[error("No exact nor similar matches found for token: {0}")]
    NoExactOrSimilarMatch(String),

    #[error("No exact matches found for token: {0}, but found similar matches: {1:?}")]
    NoExactMatchButSimilar(String, Vec<String>),

    #[error("Multiple exact matches found for token: {0} -> {1:?}, cannot parse")]
    MultipleExactMatches(String, Vec<String>),
}

fn parse_amount(amount: &str) -> error_stack::Result<f64, AaHParserError> {
    let more_than_10_zeroes_regex = regex::Regex::new(r"[₁-₉][^\d\w ]+").unwrap();

    let amount = more_than_10_zeroes_regex.replace_all(amount, "₀");

    let amount = amount
        .replace("₀", "000000000")
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

    let amount = amount.replace(",", "");

    let amount =
        amount
            .parse::<f64>()
            .change_context(AaHParserError::Parse(ParseError::Amount(format!(
                "Failed to parse amount: '{}'",
                amount
            ))))?;

    Ok(amount)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AaHLocation<'a> {
    pub chain: &'a str,
    pub custody: AaHCustody<'a>,
    pub token_name: &'a str,
}

impl<'a> AaHLocation<'a> {
    pub fn from_wallet_token(chain: &'a str, token_name: &'a str) -> AaHLocation<'a> {
        AaHLocation {
            chain,
            custody: AaHCustody::Wallet,
            token_name,
        }
    }

    pub fn from_project_tracking(
        chain: &'a str,
        project_name: &'a str,
        tracking_type: &'a str,
        balance_type: &'a str,
        token_name: &'a str,
    ) -> AaHLocation<'a> {
        AaHLocation {
            chain,
            custody: AaHCustody::Project {
                project_name,
                tracking_type,
                balance_type,
            },
            token_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AaHCustody<'a> {
    Wallet,
    Project {
        project_name: &'a str,
        tracking_type: &'a str,
        balance_type: &'a str,
    },
}

impl<'a> Display for AaHLocation<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.custody {
            AaHCustody::Wallet => write!(f, "{} - <wallet> ({})", self.chain, self.token_name),
            AaHCustody::Project {
                project_name,
                tracking_type,
                balance_type,
            } => write!(
                f,
                "{} - {}<{}, {}> ({})",
                self.chain, project_name, tracking_type, balance_type, self.token_name
            ),
        }
    }
}

impl AaHParser {
    #[instrument]
    pub fn new() -> AaHParser {
        AaHParser {
            balances: HashMap::new(),
        }
    }

    #[instrument(skip(self))]
    fn parse_generic(
        &mut self,
        token_location: AaHLocation,
        amount: &str,
        usd_value_str: Option<&str>,
        extra_names: Option<&[&str]>,
    ) -> error_stack::Result<(), AaHParserError> {
        let matches = RELEVANT_DEBANK_TOKENS
            .iter()
            .flat_map(|relevant_token: &RelevantDebankToken| {
                let main_match = (
                    relevant_token,
                    relevant_token.matches(token_location.token_name),
                );

                let extras = if let Some(extra_names) = extra_names {
                    extra_names
                        .iter()
                        .map(|extra_name| {
                            (relevant_token, relevant_token.matches(extra_name.as_ref()))
                        })
                        .collect()
                } else {
                    vec![]
                };

                vec![main_match].into_iter().chain(extras.into_iter())
            })
            .collect::<Vec<_>>();

        let relevant_tokens = matches
            .into_iter()
            .filter(|(_, token_match)| *token_match != TokenMatch::NoMatch)
            .collect::<Vec<_>>();

        let exact_matches = relevant_tokens
            .iter()
            .filter(|(_, token_match)| *token_match == TokenMatch::ExactMatch)
            .collect::<Vec<_>>();

        if exact_matches.is_empty() {
            let similar_matches = relevant_tokens
                .iter()
                .filter_map(|(_, token_match)| {
                    if let TokenMatch::SimilarMatch(similar) = token_match {
                        Some(similar)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if similar_matches.is_empty() {
                return Err(report!(AaHParserError::TokenMatch(
                    TokenMatchError::NoExactOrSimilarMatch(token_location.to_string(),)
                )));
            } else {
                return Err(report!(AaHParserError::TokenMatch(
                    TokenMatchError::NoExactMatchButSimilar(
                        token_location.to_string(),
                        similar_matches
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    )
                )));
            }
        }

        let unique_exact_match_names = exact_matches
            .iter()
            .map(|(relevant_token, _)| relevant_token.token_name)
            .collect::<HashSet<_>>();

        if unique_exact_match_names.len() > 1 {
            tracing::error!(
                "Multiple exact_matches found for token {} -> \n{:#?}, cannot parse",
                token_location,
                exact_matches
            );
            return Err(report!(AaHParserError::TokenMatch(
                TokenMatchError::MultipleExactMatches(
                    token_location.to_string(),
                    unique_exact_match_names
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                )
            )));
        }

        let Some((token, _)) = exact_matches.first() else {
            tracing::warn!(
                "Ignoring token: '{}' since it does not seem to be relevant (no exact matches found)",
                token_location,
            );
            return Ok(());
        };

        let token_balances = self
            .balances
            .entry(token.token_name.to_owned())
            .or_insert(HashMap::new());

        let mut amount = parse_amount(amount)?;

        // Parse USD value - use None when USD value is not available
        let mut usd_value = if let Some(usd_str) = usd_value_str {
            Some(format_balance(usd_str).map_err(|e| {
                AaHParserError::Parse(ParseError::Amount(format!(
                    "Failed to parse USD value: '{:?}', error: {}",
                    usd_str, e
                )))
            })?)
        } else {
            None
        };

        let name = format!("{token_location}");

        if token_balances.contains_key(&name) {
            tracing::warn!(
                "The same location has appeared multiple times: '{}', adding amounts together",
                name
            );
            let existing = token_balances.get(&name).unwrap();
            tracing::warn!(
                "Previous values for '{}': amount={}, usd_value={:?}",
                name,
                existing.amount,
                existing.usd_value
            );
            amount += existing.amount;

            // Add USD values if both are Some, otherwise keep the existing logic
            usd_value = match (usd_value, existing.usd_value) {
                (Some(new_val), Some(existing_val)) => Some(new_val + existing_val),
                (Some(new_val), None) => Some(new_val),
                (None, Some(existing_val)) => Some(existing_val),
                (None, None) => None,
            };

            tracing::warn!(
                "New values for '{}': amount={}, usd_value={:?}",
                name,
                amount,
                usd_value
            );
        }
        token_balances.insert(name, TokenBalance { amount, usd_value });
        Ok(())
    }

    #[instrument(skip(self, wallet), fields(usd_value = ?wallet.usd_value, token_count = ?wallet.tokens.len()))]
    pub fn parse_wallet(
        &mut self,
        chain: &str,
        wallet: &ChainWallet,
    ) -> error_stack::Result<(), AaHParserError> {
        for token in wallet.tokens.as_slice() {
            // Parse, return error, but ignore if error is a token match error
            let result = self.parse_generic(
                AaHLocation::from_wallet_token(chain, token.name.as_str()),
                token.amount.as_str(),
                Some(token.usd_value.as_str()),
                None,
            );

            if let Err(e) = result {
                if let AaHParserError::TokenMatch(_) = e.current_context() {
                    tracing::warn!("Ignoring token match error: {}", e);
                } else {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, token), fields(token = ?token.token_name))]
    fn parse_simple_token(
        &mut self,
        chain: &str,
        project_name: &str,
        tracking_type: &str,
        token: &SimpleTokenInfo,
    ) -> error_stack::Result<(), AaHParserError> {
        let extra_names = if let Some(token_name) = token.token_name.as_deref() {
            Some(vec![token_name])
        } else {
            None
        };

        self.parse_generic(
            AaHLocation::from_project_tracking(
                chain,
                project_name,
                tracking_type,
                "Balance",
                token.pool.as_str(),
            ),
            token.balance.as_str(),
            Some(token.usd_value.as_str()),
            extra_names.as_deref(),
        )
    }

    fn split_balance_token(s: &str) -> Option<(&str, &str)> {
        s.split_once(' ')
    }

    #[instrument(skip(self, token), fields(token = ?token.token_name))]
    fn parse_stake_shaped_token(
        &mut self,
        chain: &str,
        project_name: &str,
        tracking_type: &str,
        token: &StakeTokenInfo,
    ) -> error_stack::Result<(), AaHParserError> {
        let tokens_with_balances = token
            .balance
            .split('\n')
            .filter_map(|s| {
                let Some((balance, token_name)) = AaHParser::split_balance_token(s) else {
                    tracing::error!(
                        "Failed to split stake-like token balance and token name: '{}'. Skipping.",
                        s
                    );
                    return None;
                };

                Some(("Balance", balance, token_name))
            })
            .collect::<Vec<_>>();

        let rewards_with_balances = token
            .rewards
            .as_ref()
            .map(|rewards| {
                rewards
                    .split('\n')
                    .filter_map(|s| {
                        let Some((balance, token_name)) = AaHParser::split_balance_token(s) else {
                            tracing::error!(
                                "Failed to split rewards balance and token name: '{}'. Skipping.",
                                s
                            );
                            return None;
                        };

                        Some(("Rewards", balance, token_name))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let all_types_with_balances = tokens_with_balances
            .into_iter()
            .chain(rewards_with_balances.into_iter())
            .collect::<Vec<_>>();

        // If there's only one token balance and it matches the pool name, use the token's USD value
        // This handles simple staked tokens like ENA where the balance is a single line
        let use_token_usd_value =
            all_types_with_balances.len() == 1 && all_types_with_balances[0].2 == token.pool;

        for (balance_type, balance, token_name) in all_types_with_balances.as_slice() {
            tracing::info!(
                "Parsing stake-like token (Project: {project_name}): balance: {balance}, token_name: {token_name}, type: {balance_type}",
            );

            // Use the actual USD value for simple staked tokens, otherwise None
            let usd_value = if use_token_usd_value {
                token.usd_value.as_deref()
            } else {
                None // No USD value available for multi-token parsed entries
            };

            let result = self.parse_generic(
                AaHLocation::from_project_tracking(
                    chain,
                    project_name,
                    tracking_type,
                    balance_type,
                    token_name,
                ),
                balance,
                usd_value,
                None,
            );

            if let Err(e) = result {
                if let AaHParserError::TokenMatch(_) = e.current_context() {
                    tracing::warn!("Ignoring token match error: {}", e);
                } else {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, token), fields(token = ?token.token_name))]
    fn parse_lending_token(
        &mut self,
        chain: &str,
        project_name: &str,
        balance_type: &str,
        token: &LendingTokenInfo,
    ) -> error_stack::Result<(), AaHParserError> {
        self.parse_generic(
            AaHLocation::from_project_tracking(
                chain,
                project_name,
                "Lending",
                balance_type,
                token.token_name.as_str(),
            ),
            token.balance.as_str(),
            Some(token.usd_value.as_str()),
            None,
        )
    }

    #[instrument(skip(self, project), fields(project = ?project.name))]
    pub fn parse_project(
        &mut self,
        chain: &str,
        project: &Project,
    ) -> error_stack::Result<(), AaHParserError> {
        let project_name = project.name.clone();

        for tracking in project.trackings.as_slice() {
            let ProjectTracking {
                tracking_type,
                token_sections,
            } = tracking;

            const SIMPLE: &[&str] = ["Yield", "Deposit"].as_slice();
            const STAKE_SHAPED: &[&str] = [
                "Farming",
                "Vesting",
                "Rewards",
                "Locked",
                "Liquidity Pool",
                "Staked",
            ]
            .as_slice();

            let convert_to_simple = |token: &TokenInfo| SimpleTokenInfo {
                token_name: token.token_name.clone(),
                pool: token
                    .pool
                    .as_ref()
                    .expect("Pool should be present for simple tokens")
                    .clone(),
                balance: token
                    .balance
                    .as_ref()
                    .expect("Balance should be present for simple tokens")
                    .clone(),
                usd_value: token
                    .usd_value
                    .as_ref()
                    .expect("USD value should be present for simple tokens")
                    .clone(),
            };

            let convert_to_stake_shaped = |token: &TokenInfo| StakeTokenInfo {
                balance: token
                    .balance
                    .as_ref()
                    .expect("Balance should be present for stake-shaped tokens")
                    .clone(),
                pool: token
                    .pool
                    .as_ref()
                    .expect("Pool should be present for stake-shaped tokens")
                    .clone(),
                token_name: token.token_name.clone(),
                rewards: None,
                usd_value: token.usd_value.clone(),
            };

            let convert_to_lending = |token: &TokenInfo| LendingTokenInfo {
                token_name: token
                    .token_name
                    .as_ref()
                    .expect("Token name should be present for lending tokens")
                    .clone(),
                balance: token
                    .balance
                    .as_ref()
                    .expect("Balance should be present for lending tokens")
                    .clone(),
                usd_value: token
                    .usd_value
                    .as_ref()
                    .expect("USD value should be present for lending tokens")
                    .clone(),
            };

            for section in token_sections {
                for token in section.tokens.as_slice() {
                    let mut token = token.clone();
                    if section.title == "Borrowed" {
                        let negative_balance = token.balance.map(|balance| format!("-{balance}"));
                        token.balance = negative_balance;

                        let negative_usd_value = token.usd_value.map(|usd| format!("-{usd}"));
                        token.usd_value = negative_usd_value;
                    }

                    let result = if SIMPLE.contains(&tracking_type.as_str()) {
                        self.parse_simple_token(
                            chain,
                            project_name.as_str(),
                            tracking_type,
                            &convert_to_simple(&token),
                        )
                    } else if STAKE_SHAPED.contains(&tracking_type.as_str()) {
                        self.parse_stake_shaped_token(
                            chain,
                            project_name.as_str(),
                            tracking_type,
                            &convert_to_stake_shaped(&token),
                        )
                    } else if tracking_type == "Lending" {
                        self.parse_lending_token(
                            chain,
                            project_name.as_str(),
                            section.title.as_str(),
                            &convert_to_lending(&token),
                        )
                    } else {
                        return Err(report!(AaHParserError::UnknownTrackingType(
                            tracking_type.clone(),
                            token.clone()
                        )));
                    };

                    if let Err(e) = result {
                        if let AaHParserError::TokenMatch(_) = e.current_context() {
                            tracing::warn!("Ignoring token match error: {}", e);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
