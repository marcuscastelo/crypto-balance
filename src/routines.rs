use crate::prelude::*;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use binance::account::Account;
use binance::api::Binance;
use binance::config::Config;
use binance::market::Market;
use binance::rest_model::Prices;
use google_sheets4::api::ValueRange;
use serde_json::Value;

pub struct UpdateBinanceBalanceRoutine;
pub struct UpdateKrakenBalanceRoutine;
pub struct UpdateAirdropWalletBalanceRoutine;
pub struct UpdateTokenPricesRoutine;

async fn get_chain_balance(chain: &Chain, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
    println!("Fetching balance for {}", chain.name);

    println!("Fetching native balance for {}", chain.name);
    let native_balance = chain.explorer.fetch_native_balance(evm_address).await;
    println!("Fetching ERC20 balances for {}", chain.name);
    let erc20_balances = chain.explorer.fetch_erc20_balances(evm_address).await;
    println!("Merging balances for {}", chain.name);
    let mut balances = erc20_balances;
    balances.insert(chain.native_token.to_owned(), native_balance);
    println!("Balances fetched for {}", chain.name);

    // Remove zero balances
    balances.retain(|_, balance| balance.balance > 0.0);
    balances
}

impl UpdateAirdropWalletBalanceRoutine {
    pub async fn run(&self) {
        println!("Starting...");
        let evm_address = &CONFIG.blockchain.evm_address;

        let sheet_title = "Balance - Airdrop Wallet";
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        println!("Creating futures...");
        let futures = CHAINS
            .values()
            .map(|chain| async { (chain.name, get_chain_balance(chain, evm_address).await) });

        println!("Waiting for futures...");
        let a = futures::future::join_all(futures).await;
        println!("Futures done...");
        let chain_balances = a.into_iter().collect::<HashMap<_, _>>();
        println!("Chain balances: {:#?}", chain_balances);

        println!("Creating unique tokens...");

        // Create a set of unique token structs using their names as keys
        let mut unique_tokens: HashMap<String, Arc<Token>> = HashMap::new();
        for (_, balances) in &chain_balances {
            for (token, _) in balances {
                match token.as_ref() {
                    Token::Native(token_name) => {
                        unique_tokens.insert(token_name.as_str().to_owned(), token.clone());
                    }
                    Token::ERC20(token_info) => {
                        unique_tokens.insert(token_info.token_symbol.to_string(), token.clone());
                    }
                }
            }
        }
        let mut unique_tokens = unique_tokens.into_iter().collect::<Vec<_>>();
        unique_tokens.sort_by(|a, b| a.0.cmp(&b.0));
        let unique_tokens = unique_tokens;

        println!("Writing token names...");

        // Write the token names to the spreadsheet (B3:B1000)
        spreadsheet_manager
            .write_range(
                format!("'{}'!B3:B1000", sheet_title).as_str(),
                ValueRange::from_rows(
                    unique_tokens
                        .iter()
                        .map(|(_, token)| match token.as_ref() {
                            Token::Native(token_name) => token_name.as_str(),
                            Token::ERC20(token_info) => token_info.token_symbol.as_ref(),
                        })
                        .collect::<Vec<_>>()
                        .as_ref(),
                ),
            )
            .await
            .unwrap();

        let mut chain_names = chain_balances.keys().cloned().collect::<Vec<_>>();
        chain_names.sort();
        let chain_names = chain_names;

        println!("Writing token names done!");

        let start_letter = 'C';
        let mut current_chain_idx = 0; // Skip the first column for the token names
        for chain in chain_names {
            println!("Writing balances for {}", chain);

            spreadsheet_manager
                .write_range(
                    format!(
                        "'{}'!{}2",
                        sheet_title,
                        (start_letter as u8 + current_chain_idx) as char
                    )
                    .as_str(),
                    ValueRange::from_str(chain),
                )
                .await
                .unwrap();

            let mut token_balances = Vec::with_capacity(unique_tokens.len());
            for (_, token) in &unique_tokens {
                token_balances.push(
                    chain_balances
                        .get(chain)
                        .unwrap_or_else(|| panic!("Chain {} should have balance", chain))
                        .get(token)
                        .map(|x| x.balance.to_string())
                        .unwrap_or("".to_owned()),
                );
            }

            let current_letter = (start_letter as u8 + current_chain_idx) as char;
            current_chain_idx += 1;

            let range = format!(
                "'{}'!{}3:{}{}",
                sheet_title,
                current_letter,
                current_letter,
                4 + token_balances.len()
            );

            println!("Writing to range: {}", range);
            spreadsheet_manager
                .write_range(
                    range.as_str(),
                    ValueRange::from_rows(
                        token_balances
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .as_ref(),
                    ),
                )
                .await
                .unwrap();

            println!("Writing balances for {} done!", chain);
            println!(
                "Written: {:?}",
                ValueRange::from_rows(
                    token_balances
                        .iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .as_ref(),
                )
            );
        }
        println!("Writing balances done!");
    }
}

impl UpdateBinanceBalanceRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let binance_account: Account = Binance::new_with_config(
            Some(CONFIG.binance.api_key.to_string()),
            Some(CONFIG.binance.secret_key.to_string()),
            &Config {
                rest_api_endpoint: "https://api.binance.com".into(),
                ws_endpoint: "wss://stream.binance.com:9443".into(),

                futures_rest_api_endpoint: "https://fapi.binance.com".into(),
                futures_ws_endpoint: "wss://fstream.binance.com".into(),

                recv_window: 50000,
                binance_us_api: false,

                timeout: None,
            },
        );

        println!(
            "Binance account: {:#?}",
            binance_account
                .get_account()
                .await
                .unwrap()
                .balances
                .into_iter()
                .filter(|x| x.free > 0.0)
                .collect::<Vec<_>>()
        );

        let token_names: Vec<String> = spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into();

        let balances = binance_account
            .get_account()
            .await
            .unwrap()
            .balances
            .into_iter()
            .filter(|x| x.free > 0.0)
            // Convert to Hashmap of token.asset, token.free
            .map(|token| (token.asset, token.free))
            .collect::<HashMap<_, _>>();

        // Write to the spreadsheet
        let mut token_balances = Vec::with_capacity(token_names.len());
        for token_name in &token_names {
            token_balances.push(balances.get(token_name).unwrap_or(&0.0));
        }

        println!(
            "Balances in order:\n{:#?}",
            token_names
                .iter()
                .zip(token_balances.clone())
                .collect::<Vec<_>>()
        );

        spreadsheet_manager
            .write_named_range(
                ranges::balances::binance::RW_AMOUNTS,
                // TODO: create Vec<T> to ValueRange conversion
                ValueRange {
                    range: None,
                    major_dimension: None,
                    values: Some(
                        token_balances
                            .into_iter()
                            .map(|balance| {
                                vec![Value::Number(
                                    serde_json::Number::from_f64(*balance).unwrap(),
                                )]
                            })
                            .collect::<Vec<_>>(),
                    ),
                },
            )
            .await
            .expect("Should write balances to the spreadsheet");
    }
}

impl UpdateKrakenBalanceRoutine {
    pub async fn run(&self) {
        unimplemented!("Kraken balance update routine is not implemented");
    }
}

impl UpdateTokenPricesRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let token_names: Vec<String> = spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into();

        let normal_pairs = token_names
            .clone()
            .into_iter()
            .map(|name| (name.clone(), format!("{}USDT", name.clone())))
            .collect::<Vec<(String, String)>>();

        let reverse_pairs = token_names
            .clone()
            .into_iter()
            .map(|name| (name.clone(), format!("USDT{}", name.clone())))
            .collect::<Vec<(String, String)>>();

        let binance_market: Market = Binance::new(None, None);
        let Prices::AllPrices(all_prices) = binance_market
            .get_all_prices()
            .await
            .expect("Should get all prices from Binance API");

        let mut prices: HashMap<_, _> =
            HashMap::with_capacity(normal_pairs.len() + reverse_pairs.len());

        // TODO: Move USDT mentions to a constant for easier maintenance
        // Special case for USDT/USDT pair that doesn't exist since it wouldn't make sense
        prices.insert("USDT".to_string(), 1.0);

        for symbol_price in all_prices {
            if let Some(normal_pair) = normal_pairs.iter().find(|x| x.1 == symbol_price.symbol) {
                prices.insert(normal_pair.0.clone(), symbol_price.price);
            }

            if let Some(reverse_pair) = reverse_pairs.iter().find(|x| x.1 == symbol_price.symbol) {
                prices.insert(reverse_pair.0.clone(), 1.0 / symbol_price.price);
            }
        }

        let mut token_prices_in_order = Vec::with_capacity(token_names.len());
        for token_name in &token_names {
            token_prices_in_order.push(prices.get(token_name).unwrap_or_else(|| {
                println!("Warning: No price for {}", token_name);
                &0.0
            }));
        }

        println!(
            "{:?}",
            token_names
                .iter()
                .zip(token_prices_in_order.clone())
                .collect::<Vec<_>>()
        );

        let a = ValueRange {
            range: None,
            major_dimension: None,
            values: Some(
                token_prices_in_order
                    .into_iter()
                    .map(|price| vec![Value::Number(serde_json::Number::from_f64(*price).unwrap())])
                    .collect::<Vec<_>>(),
            ),
        };

        spreadsheet_manager
            .write_named_range(ranges::tokens::RW_PRICES, a)
            .await
            .expect("Should write prices to the spreadsheet");
    }
}
