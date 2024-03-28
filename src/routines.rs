use crate::exchange::binance::factory::BinanceAccountFactory;
use crate::prelude::*;
use crate::user_addresses::UserAddresses;

use std::collections::HashMap;

use std::sync::Arc;

use ::binance::account::Account;
use ::binance::api::Binance;
use ::binance::config::Config;
use ::binance::market::Market;
use ::binance::rest_model::Prices;
use google_sheets4::api::ValueRange;
use num_traits::ToPrimitive;
use serde_json::Value;

pub struct SheetsGetTokenNamesRoutine;
pub struct SheetsGetTokenIDsRoutine;
pub struct FetchEvmChainBalancesRoutine;
pub struct FetchCosmosChainBalancesRoutine;

// Hold Balances tabs
pub struct UpdateBinanceBalanceOnSheetsRoutine;
pub struct UpdateKrakenBalanceOnSheetsRoutine;
// TODO: decide where Airdrop Wallet will be (remove from here?)
pub struct UpdateAirdropWalletOnSheetsBalanceRoutine;
pub struct UpdateTokenPricesOnSheetsViaBinanceRoutine;

// Airdrop tab
pub struct UpdateAirdropDebankTotalOnSheetsRoutine;
pub struct UpdateAirdropStepSVMTotalOnSheetsRoutine;

pub struct UpdateTokenPricesOnSheetsViaCoinGeckoRoutine;

impl SheetsGetTokenNamesRoutine {
    pub async fn run(&self) -> Vec<String> {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }
}

impl SheetsGetTokenIDsRoutine {
    pub async fn run(&self) -> Vec<String> {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_IDS)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }
}

impl FetchEvmChainBalancesRoutine {
    async fn run(&self, chain: &Chain, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
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
}

impl FetchCosmosChainBalancesRoutine {
    async fn run(&self, chain: &Chain, cosmos_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        println!("Fetching balance for {}", chain.name);

        println!("Fetching native balance for {}", chain.name);
        let native_balance = chain.explorer.fetch_native_balance(cosmos_address).await;

        // println!("Fetching IBC balances for {}", chain.name);
        // let ibc_balances = chain.explorer.fetch_ibc_balances(cosmos_address).await;

        println!("Merging balances for {}", chain.name);

        let mut balances = HashMap::new();
        balances.insert(chain.native_token.to_owned(), native_balance);
        // balances.extend(ibc_balances);
        println!("Balances fetched for {}", chain.name);

        // Remove zero balances
        balances.retain(|_, balance| balance.balance > 0.0);
        balances
    }
}

impl UpdateAirdropWalletOnSheetsBalanceRoutine {
    pub async fn run(&self) {
        let user_addresses = UserAddresses::from_config(&CONFIG.blockchain);
        let sheet_title = "Balance per Chain - Airdrop Wallet";

        let evm_chain_balance_routines = EVM_CHAINS.values().map(|chain| async {
            (
                chain.name,
                FetchEvmChainBalancesRoutine
                    .run(chain, &CONFIG.blockchain.evm.address)
                    .await,
            )
        });

        let cosmos_chain_balance_routines = COSMOS_CHAINS.values().map(|chain| async {
            (
                chain.name,
                FetchCosmosChainBalancesRoutine
                    .run(
                        chain,
                        user_addresses
                            .get_addresses(chain)
                            .unwrap()
                            .first()
                            .unwrap(),
                    )
                    .await,
            )
        });

        let mut chain_balances: HashMap<&str, HashMap<Arc<Token>, TokenBalance>> = HashMap::new();

        chain_balances.extend(
            futures::future::join_all(cosmos_chain_balance_routines)
                .await
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );

        chain_balances.extend(
            futures::future::join_all(evm_chain_balance_routines)
                .await
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );

        println!("Chain balances: {:#?}", chain_balances);

        println!("Starting sheet manipulation...");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        println!("Creating unique tokens...");

        // Create a set of unique token structs using their names as keys
        let mut unique_tokens: HashMap<String, Arc<Token>> = HashMap::new();
        for balances in chain_balances.values() {
            for token in balances.keys() {
                match token.as_ref() {
                    Token::Native(token_name) => {
                        unique_tokens.insert(token_name.to_string(), token.clone());
                    }
                    Token::ERC20(token_info) => {
                        unique_tokens.insert(token_info.token_symbol.to_string(), token.clone());
                    }
                    Token::IBC => todo!("IBC token not implemented yet"),
                }
            }
        }
        let mut unique_tokens = unique_tokens.into_iter().collect::<Vec<_>>();
        unique_tokens.sort_by(|a, b| a.0.cmp(&b.0));
        let unique_tokens = unique_tokens;

        println!("Writing token names...");
        let token_names = unique_tokens
            .iter()
            .map(|(_, token)| match token.as_ref() {
                Token::Native(token_name) => token_name.to_string(),
                Token::ERC20(token_info) => token_info.token_symbol.to_string(),
                Token::IBC => todo!("IBC token not implemented yet"),
            })
            .collect::<Vec<_>>();

        // Write the token names to the spreadsheet (B3:B1000)
        spreadsheet_manager
            .write_range(
                format!("'{}'!B3:B1000", sheet_title).as_str(),
                ValueRange::from_rows(token_names.as_ref()),
            )
            .await
            .expect("Should write token names to the spreadsheet");

        let mut chain_names = chain_balances.keys().cloned().collect::<Vec<_>>();
        chain_names.sort();
        let chain_names = chain_names;

        println!("Writing token names done!");

        let starting_col: Column = "C".try_into().unwrap();
        for (current_chain_idx, chain_name) in chain_names.iter().enumerate() {
            println!("Writing balances for {}", chain_name);

            let current_col = starting_col + current_chain_idx as u32;
            let chain_name_position = CellPosition {
                row: 2_usize.into(),
                col: current_col,
            };

            spreadsheet_manager
                .write_range(
                    chain_name_position
                        .to_a1_notation(Some(sheet_title))
                        .as_ref(),
                    ValueRange::from_str(chain_name),
                )
                .await
                .unwrap();

            let mut token_balances = Vec::with_capacity(unique_tokens.len());
            for (_, token) in &unique_tokens {
                token_balances.push(
                    chain_balances
                        .get(chain_name)
                        .unwrap_or_else(|| panic!("Chain {} should have balance", chain_name))
                        .get(token)
                        .map(|x| x.balance.to_string())
                        .unwrap_or("".to_owned()),
                );
            }

            let pivot: CellPosition = (current_col, 3_u32).into();

            let balances_range = CellRange {
                start: pivot.clone(),
                end: (pivot.col, pivot.row + token_balances.len()).into(),
            };

            println!("Writing to range: {:#?}", balances_range);
            spreadsheet_manager
                .write_range(
                    balances_range.to_a1_notation(Some(sheet_title)).as_ref(),
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

            println!("Writing balances for {} done!", chain_name);
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

impl UpdateBinanceBalanceOnSheetsRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let binance_account: Account = BinanceAccountFactory::create();

        println!(
            "Binance account balance: {:#?}",
            binance_account
                .get_account()
                .await
                .unwrap()
                .balances
                .into_iter()
                .filter(|x| x.free > 0.0)
                .collect::<Vec<_>>()
        );

        let token_names: Vec<String> = SheetsGetTokenNamesRoutine.run().await;

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

impl UpdateKrakenBalanceOnSheetsRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let kraken_api = KrakenFactory::create();

        let token_names: Vec<String> = SheetsGetTokenNamesRoutine.run().await;

        let balances = kraken_api
            .get_account_balance()
            .await
            .unwrap()
            .into_iter()
            .map(|(symbol, amount)| {
                (
                    match symbol.as_str() {
                        "XXBT" => "BTC".to_string(),
                        "XETH" => "ETH".to_string(),
                        "XXRP" => "XRP".to_string(),
                        "USD" => "USDT".to_string(),
                        _ => symbol,
                    },
                    amount.to_f64().expect("Should be convertible to f64"),
                )
            })
            .filter(|(_, amount)| *amount > 0.0)
            .collect::<HashMap<_, _>>();

            println!("Kraken account balance: {:#?}", balances);

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
                ranges::balances::kraken::RW_AMOUNTS,
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

impl UpdateTokenPricesOnSheetsViaBinanceRoutine {
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

impl UpdateTokenPricesOnSheetsViaCoinGeckoRoutine {
    pub async fn run(&self) {
        // Below: routine to get native token prices from CoinGecko (failed attempt)
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let token_ids: Vec<String> = SheetsGetTokenIDsRoutine.run().await;

        // let coins = CoinGeckoApi.list_coins().await;
        // let coin_tuples = coins
        //     .into_iter()
        //     .filter(|coin| token_ids.contains(&coin.id))
        //     .map(|coin| (coin.id, coin.symbol.to_uppercase()))
        //     .collect::<Vec<_>>();
        let prices = CoinGeckoApi.prices(token_ids.as_ref()).await;

        let current_prices_on_sheet = spreadsheet_manager
            .read_named_range(ranges::tokens::RW_PRICES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
            .into_iter()
            .map(|x| {
                x.replace(['$', ','], "")
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Should be a number: {}", x))
            })
            .collect::<Vec<_>>();

        let new_prices = token_ids
            .iter()
            .enumerate()
            .map(|(idx, token)| match prices.0.get(token) {
                Some(price) => price
                    .usd
                    .expect("Should have price when PriceResponse exists"),
                None => current_prices_on_sheet.get(idx).copied().unwrap_or(0.0),
            })
            .map(|price| price.to_string())
            .collect::<Vec<_>>();

        spreadsheet_manager
            .write_named_range(
                ranges::tokens::RW_PRICES,
                ValueRange::from_rows(new_prices.as_ref()),
            )
            .await
            .expect("Should write prices to the spreadsheet");
    }
}

impl UpdateAirdropDebankTotalOnSheetsRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = DebankScraper
            .get_total_balance(&CONFIG.blockchain.evm.address)
            .await
            .unwrap();

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_DEBANK_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write Debank total to the spreadsheet");
    }
}

impl UpdateAirdropStepSVMTotalOnSheetsRoutine {
    pub async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let balance = StepSVMScraper
            .get_total_balance(&CONFIG.blockchain.solana.address)
            .await
            .unwrap();

        spreadsheet_manager
            .write_named_range(
                ranges::airdrops::RW_STEP_SVM_TOTAL_USD,
                ValueRange::from_str(&balance.to_string()),
            )
            .await
            .expect("Should write StepSVM total to the spreadsheet");
    }
}
