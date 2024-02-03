#![feature(lazy_cell)]

mod app_config;
mod blockchain;
mod exchange;
mod prelude;
mod sheets;

use std::collections::HashMap;
use std::sync::Arc;

use google_sheets4::Sheets;

use crate::prelude::*;

fn get_chain_balance(chain: &Chain, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
    let native_balance = chain.explorer.fetch_native_balance(evm_address);
    let erc20_balances = chain.explorer.fetch_erc20_balances(evm_address);
    let mut balances = erc20_balances;
    balances.insert(chain.native_token.to_owned(), native_balance);
    balances
}

#[tokio::main]
async fn main() {
    let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

    match spreadsheet_manager.named_range_map().await {
        Some(named_ranges) => {
            for (name, range) in named_ranges {
                println!(
                    "{:?}: {:?}",
                    name,
                    range.to_a1_notation(
                        spreadsheet_manager
                            .get_sheet_title(range.sheet_id.unwrap())
                            .await
                            .unwrap()
                            .as_str()
                    )
                );
            }
        }
        None => {
            println!("No named ranges found");
        }
    }

    // let binance: Market = Binance::new(None, None);

    // println!("BTC Price: {:?}", binance.get_price("BTCUSDT").unwrap());

    // for network in NETWORKS.values() {
    //     println!("{} --------------", network.name);

    //     let network_balances = get_network_balance(network, &CONFIG.blockchain.evm_address);

    //     for (token, balance) in network_balances {
    //         match token.as_ref() {
    //             Token::Native(token_name) => {
    //                 println!("   {} {:?}", balance.balance, token_name);
    //             }
    //             Token::ERC20(token_info) => {
    //                 println!(
    //                     "   {} {} ({})",
    //                     balance.balance, token_info.token_symbol, token_info.token_name
    //                 );
    //             }
    //         }
    //     }

    //     println!("   --------------");
    //     println!();
    //     println!();
    // }
}
