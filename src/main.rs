#![feature(lazy_cell)]

mod app_config;
mod blockchain;
mod exchange;
mod prelude;
mod price;
mod routines;
mod sheets;

use prelude::block_explorer::explorers::mintscan::mintscan_implementation::Mintscan;

use crate::prelude::*;

#[tokio::main]
async fn main() {
    let test = Mintscan
        .fetch_native_balance(&app_config::CONFIG.blockchain.cosmos.celestia_address)
        .await;

    println!("{:#?}", test);

    // routines::UpdateAirdropWalletOnSheetsBalanceRoutine
    //     .run()
    //     .await
}
