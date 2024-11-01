use std::collections::HashMap;

use crate::routines::debank_tokens_routine::{RelevantDebankToken, RELEVANT_DEBANK_TOKENS};

use super::debank_scraper::{
    ChainProjectInfo, ChainWalletInfo, DepositTokenInfo, ProjectTracking, SpotTokenInfo,
    StakeTokenInfo, YieldFarmTokenInfo,
};

pub struct AaHParser {
    pub balances: HashMap<String, HashMap<String, f64>>,
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

impl AaHParser {
    pub fn new() -> AaHParser {
        AaHParser {
            balances: HashMap::new(),
        }
    }

    fn parse_generic(
        &mut self,
        chain: &str,
        location: &str,
        amount: &str,
        token: &str,
        relevant_tokens: &[&RelevantDebankToken],
    ) -> anyhow::Result<()> {
        if relevant_tokens.len() > 1 {
            log::error!(
                "Multiple relevant tokens found for token: {}. Halt!.",
                token
            );
            return Err(anyhow::anyhow!("Multiple relevant tokens found"));
        }

        if relevant_tokens.is_empty() {
            log::warn!("Ignoring token: {}", token);
            return Ok(());
        }

        let relevant_token = relevant_tokens.first().unwrap();
        let token_balances = self
            .balances
            .entry(relevant_token.token_name.to_owned())
            .or_insert(HashMap::new());

        let amount = parse_amount(amount)?;
        let name = format!("{}@{} ({})", location, chain, token);

        token_balances.insert(name, amount);
        Ok(())
    }

    pub fn parse_wallet(&mut self, chain: &str, wallet: &ChainWalletInfo) {
        for token in wallet.tokens.as_slice() {
            let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
                .iter()
                .filter(|relevant_token| relevant_token.matches(&token.name))
                .collect::<Vec<_>>();

            self.parse_generic(
                chain,
                "Wallet",
                token.amount.as_str(),
                token.name.as_str(),
                matching_relevant_tokens.as_slice(),
            )
            .unwrap_or_else(|error| {
                log::error!(
                    "Failed to parse wallet token: {}. Error: {:?}",
                    token.name,
                    error
                );
            })
        }
    }

    fn parse_yield_token(&mut self, chain: &str, project_name: &str, token: &YieldFarmTokenInfo) {
        let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
            .iter()
            .filter(|relevant_token| {
                let matches_pool = relevant_token.matches(&token.pool);
                let matches_token_name = if let Some(token_name) = token.token_name.as_ref() {
                    relevant_token.matches(token_name)
                } else {
                    false
                };

                matches_pool || matches_token_name
            })
            .collect::<Vec<_>>();

        self.parse_generic(
            chain,
            project_name,
            token.balance.as_str(),
            token.pool.as_str(),
            matching_relevant_tokens.as_slice(),
        )
        .unwrap_or_else(|error| {
            log::error!(
                "Failed to parse yield farm token: {}. Error: {:?}",
                token.pool,
                error
            );
        });
    }

    fn parse_deposit_token(&mut self, chain: &str, project_name: &str, token: &DepositTokenInfo) {
        let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
            .iter()
            .filter(|relevant_token| {
                let matches_pool = relevant_token.matches(&token.pool);
                let matches_token_name = if let Some(token_name) = token.token_name.as_ref() {
                    relevant_token.matches(token_name)
                } else {
                    false
                };

                matches_pool || matches_token_name
            })
            .collect::<Vec<_>>();

        self.parse_generic(
            chain,
            project_name,
            token.balance.as_str(),
            token.pool.as_str(),
            matching_relevant_tokens.as_slice(),
        )
        .unwrap_or_else(|error| {
            log::error!(
                "Failed to parse deposit token: {}. Error: {:?}",
                token.pool,
                error
            );
        });
    }
    fn parse_stake_token(&mut self, chain: &str, project_name: &str, token: &StakeTokenInfo) {
        let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
            .iter()
            .filter(|relevant_token| {
                let matches_pool = relevant_token.matches(&token.pool);
                let matches_token_name = if let Some(token_name) = token.token_name.as_ref() {
                    relevant_token.matches(token_name)
                } else {
                    false
                };

                matches_pool || matches_token_name
            })
            .collect::<Vec<_>>();

        self.parse_generic(
            chain,
            project_name,
            token.balance.as_str(),
            token.pool.as_str(),
            matching_relevant_tokens.as_slice(),
        )
        .unwrap_or_else(|error| {
            log::error!(
                "Failed to parse stake token: {}. Error: {:?}",
                token.pool,
                error
            );
        });
    }

    pub fn parse_project(&mut self, chain: &str, project: &ChainProjectInfo) {
        let project_name = project.name.clone();

        for tracking in project.trackings.as_slice() {
            match tracking {
                ProjectTracking::YieldFarm { yield_farm } => {
                    for token in yield_farm {
                        self.parse_yield_token(chain, project_name.as_str(), token);
                    }
                }
                ProjectTracking::Staked { staked } => {
                    for token in staked {
                        self.parse_stake_token(chain, project_name.as_str(), token);
                    }
                }
                ProjectTracking::Deposit { deposit } => {
                    for token in deposit {
                        self.parse_deposit_token(chain, project_name.as_str(), token);
                    }
                }
                _ => {
                    log::error!("Unsupported tracking: {:?}", tracking);
                }
            }
        }
    }
}
