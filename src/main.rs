#![feature(lazy_cell)]
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
    futures::join!(
        routines::UpdateAirdropStepSVMTotalOnSheetsRoutine.run(),
        routines::UpdateAirdropDebankTotalOnSheetsRoutine.run(),
        routines::UpdateTokenPricesOnSheetsViaCoinGeckoRoutine.run(),
        routines::UpdateBinanceBalanceOnSheetsRoutine.run(),
        routines::UpdateBybitBalanceOnSheetsRoutine.run(),
        routines::UpdateKrakenBalanceOnSheetsRoutine.run(),
    );

    // Kill all geckodriver processes
    let _ = Command::new("pkill").arg("geckodriver").output();
}
