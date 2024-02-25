#![feature(lazy_cell)]

mod app_config;
mod blockchain;
mod exchange;
mod prelude;
mod price;
mod routines;
mod sheets;

use coingecko::prelude::CoinGeckoApi;

use crate::prelude::*;

#[tokio::main]
async fn main() {
    routines::UpdateTokenPricesOnSheetsViaCoinGeckoRoutine
        .run()
        .await;

    routines::UpdateBinanceBalanceOnSheetsRoutine.run().await;

    return;

    // Below: routine to get native token prices from CoinGecko (failed attempt)
    let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

    let tokens = spreadsheet_manager
        .read_range("'Prices per Chain - Airdrop Wallet'!B3:B1000")
        .await
        .unwrap()
        .values
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value[0]
                        .clone()
                        .to_string()
                        .replace('\"', "")
                        .to_uppercase()
                })
                .collect::<Vec<String>>()
        })
        .unwrap();

    let coins = CoinGeckoApi.list_coins().await;
    let mut coin_ids = coins
        .into_iter()
        .map(|coin| (coin.id, coin.symbol.to_uppercase()))
        .collect::<Vec<_>>();
    coin_ids.sort_by(|a, b| a.1.cmp(&b.1));

    // let prices = CoinGeckoApi
    //     .prices(coins.into_iter().map(|c| c.id).collect())
    //     .await;

    for coin_id in coin_ids {
        if tokens.contains(&coin_id.1) {
            println!("{:?}", coin_id);
        }
    }
    // routines::UpdateAirdropWalletOnSheetsBalanceRoutine
    //     .run()
    //     .await
}
