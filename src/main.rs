#![feature(once_cell)]
#![feature(async_closure)]

mod app_config;
mod blockchain;
mod exchange;
mod prelude;
mod price;
mod routines;
mod scraping;
mod sheets;

use coingecko::prelude::CoinGeckoApi;
use tokio::process::Command;

use crate::prelude::*;

#[tokio::main]
async fn main() {
    routines::UpdateBybitBalanceOnSheetsRoutine.run().await;
    return;

    futures::join!(
        routines::UpdateAirdropStepSVMTotalOnSheetsRoutine.run(),
        routines::UpdateAirdropDebankTotalOnSheetsRoutine.run(),
        routines::UpdateTokenPricesOnSheetsViaCoinGeckoRoutine.run(),
        routines::UpdateBinanceBalanceOnSheetsRoutine.run(),
        routines::UpdateKrakenBalanceOnSheetsRoutine.run(),
    );

    // Kill all geckodriver processes
    let _ = Command::new("pkill").arg("geckodriver").output();

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
