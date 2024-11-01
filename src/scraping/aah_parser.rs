use std::collections::HashMap;

use crate::routines::debank_tokens_routine::{RelevantDebankToken, RELEVANT_DEBANK_TOKENS};

use super::debank_scraper::{
    ChainProjectInfo, ChainWalletInfo, DepositTokenInfo, LendingTokenInfo, LiquidityPoolTokenInfo,
    ProjectTracking, SpotTokenInfo, StakeTokenInfo, YieldFarmTokenInfo,
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
        log::trace!(
            "Generic parsing: {} - {} - {} - {:?}",
            chain,
            location,
            amount,
            token
        );
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
        let name = format!("{} - {} ({})", chain, location, token);

        if token_balances.contains_key(&name) {
            log::warn!("Duplicate token found: {}. Halt!", name);
            return Err(anyhow::anyhow!("Duplicate token found"));
        }
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
            format!("{}(Yield)", project_name).as_str(),
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
            format!("{}(Deposit)", project_name).as_str(),
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

    fn parse_liquidity_pool_token(
        &mut self,
        chain: &str,
        project_name: &str,
        token: &LiquidityPoolTokenInfo,
    ) {
        let (balance1, balance2) = token
            .balance
            .split_once('\n')
            .map_or((token.balance.as_str(), None), |(balance1, balance2)| {
                (balance1, Some(balance2))
            });

        let (balance1, token1) = balance1.split_once(' ').unwrap_or((balance1, ""));
        let (balance2, token2) = balance2.map_or((None, None), |balance2| {
            let (balance2, token2) = balance2.split_once(' ').unwrap_or((balance2, ""));
            (Some(balance2), Some(token2))
        });

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
            format!("{}(Liquidity Pool: {})", project_name, token1).as_str(),
            balance1,
            token1,
            matching_relevant_tokens.as_slice(),
        )
        .unwrap_or_else(|error| {
            log::error!(
                "Failed to parse liquidity pool token: {}. Error: {:?}",
                token1,
                error
            );
        });

        if let (Some(balance2), Some(token2)) = (balance2, token2) {
            self.parse_generic(
                chain,
                format!("{}(Liquidity Pool: {})", project_name, token2).as_str(),
                balance2,
                token2,
                matching_relevant_tokens.as_slice(),
            )
            .unwrap_or_else(|error| {
                log::error!(
                    "Failed to parse liquidity pool token: {}. Error: {:?}",
                    token2,
                    error
                );
            });
        }
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
            format!("{}(Stake)", project_name).as_str(),
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

    fn parse_lending_token(&mut self, chain: &str, project_name: &str, token: &LendingTokenInfo) {
        let matching_relevant_tokens = RELEVANT_DEBANK_TOKENS
            .iter()
            .filter(|relevant_token| {
                let matches_token_name = relevant_token.matches(&token.token_name);
                matches_token_name
            })
            .collect::<Vec<_>>();

        self.parse_generic(
            chain,
            format!("{}(Lending)", project_name).as_str(),
            token.balance.as_str(),
            token.token_name.as_str(),
            matching_relevant_tokens.as_slice(),
        )
        .unwrap_or_else(|error| {
            log::error!(
                "Failed to parse lending token: {}. Error: {:?}",
                token.token_name,
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
                ProjectTracking::LiquidityPool { liquidity_pool } => {
                    for token in liquidity_pool {
                        self.parse_liquidity_pool_token(chain, project_name.as_str(), token);
                    }
                }
                ProjectTracking::Lending {
                    supplied,
                    borrowed,
                    rewards,
                } => {
                    for supplied_token in supplied.as_slice() {
                        self.parse_lending_token(
                            chain,
                            format!("{}(Supplied)", project_name).as_str(),
                            supplied_token,
                        );
                    }

                    log::debug!("{} - Borrowed: {:#?}", project.name, borrowed);
                    if let Some(borrowed_tokens) = borrowed.as_ref() {
                        for borrowed_token in borrowed_tokens.as_slice() {
                            let mut borrowed_token = borrowed_token.clone();
                            borrowed_token.balance =
                                format!("-{}", borrowed_token.balance.as_str());
                            self.parse_lending_token(
                                chain,
                                format!("{}(Borrowed)", project_name).as_str(),
                                &borrowed_token,
                            );
                        }
                    }
                }
                _ => {
                    log::error!("Unsupported tracking: {:?}", tracking);
                }
            }
        }
    }
}
