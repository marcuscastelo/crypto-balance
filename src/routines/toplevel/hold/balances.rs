use std::{collections::HashMap, sync::Arc};

use regex::Regex;

use crate::prelude::*;

use self::routines::blockchain::FetchEvmChainBalancesRoutine;

pub struct FetchHoldBalances;

fn translate_aave_supply_token(token: &str) -> (String, bool) {
    let aave_regex = Regex::new(r"^a(?:Opt)?(\w+)$").unwrap();
    match aave_regex.captures(token) {
        None => return (token.to_owned(), false),
        Some(captures) => return (captures.get(1).unwrap().as_str().to_owned(), true),
    }
}

impl FetchHoldBalances {
    pub async fn run(&self) -> HashMap<String, TokenBalance<String>> {
        let chains = vec![&POLYGON, &OPTIMISM, &ARBITRUM];

        //Parallelize fetching balances from multiple chains
        let tasks = chains.into_iter().map(|chain| {
            let routine = FetchEvmChainBalancesRoutine;
            async move {
                let hold_balances = routine
                    .run(chain, &CONFIG.blockchain.hold.evm.address)
                    .await
                    .expect(
                        format!("Should fetch '{}' chain balances for hold", chain.name).as_str(),
                    );

                let hold_sc_balances = routine
                    .run(chain, &CONFIG.blockchain.hold_sc.evm.address)
                    .await
                    .expect(
                        format!("Should fetch '{}' chain balances for hold_sc", chain.name)
                            .as_str(),
                    );

                // Merge the balances for the hold and hold_sc addresses, adding up the balances for each token
                hold_balances
                    .into_iter()
                    .chain(hold_sc_balances.into_iter())
                    .fold(HashMap::new(), |mut acc, (token, entry)| {
                        let (translated_symbol, was_aave) =
                            translate_aave_supply_token(token.symbol().as_str());

                        let mul = match translated_symbol.as_str() {
                            "WBTC" => 1e10,
                            _ => 1f64,
                        };

                        let acc_entry = acc.entry(translated_symbol.clone()).or_insert(
                            TokenBalance::<String> {
                                symbol: translated_symbol,
                                balance: 0f64,
                            },
                        );

                        acc_entry.balance += entry.balance * mul;
                        log::info!("{}: {}", acc_entry.symbol, acc_entry.balance);
                        acc
                    })
            }
        });

        let hashmaps = futures::future::join_all(tasks).await;

        //Merge all the hashmaps into one, adding up the balances for each token
        hashmaps
            .into_iter()
            .fold(HashMap::new(), |mut acc, hashmap| {
                for (token, entry) in hashmap {
                    let acc_entry = acc.entry(token).or_insert_with_key(|token| TokenBalance::<
                        String,
                    > {
                        symbol: token.clone(),
                        balance: 0f64,
                    });

                    acc_entry.balance += entry.balance;
                }
                acc
            })

        // Translate Aave supply tokens to their underlying tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_aave_supply_token() {
        assert_eq!(
            translate_aave_supply_token("aUSDC"),
            ("USDC".to_owned(), true)
        );
        assert_eq!(
            translate_aave_supply_token("aOptUSDC"),
            ("USDC".to_owned(), true)
        );
        assert_eq!(
            translate_aave_supply_token("aOptBTC"),
            ("BTC".to_owned(), true)
        );
        assert_eq!(
            translate_aave_supply_token("USDT"),
            ("USDT".to_owned(), false)
        );
        assert_eq!(
            translate_aave_supply_token("BTC"),
            ("BTC".to_owned(), false)
        );
    }
}
