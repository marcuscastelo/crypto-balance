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
    let config = &app_config::CONFIG.sheets;
    let client = http_client::http_client();
    let auth = auth::auth(config, client.clone()).await;
    let hub = Sheets::new(client.clone(), auth);

    let result = sheets::sheets::read(&hub, config).await;

    match result {
        Err(e) => println!("{}", e),
        Ok((_, spreadsheet)) => {
            let totals = Vec::new();

            println!(
                "Success: {:?}",
                spreadsheet
                    .values
                    .unwrap()
                    .into_iter()
                    .fold(totals, |mut acc, next_row| {
                        let key: String = next_row[0].to_string();
                        acc.push(key);
                        acc
                    })
            );
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
