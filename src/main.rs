#![feature(lazy_cell)]

mod app_config;
mod blockchain;
mod exchange;
mod prelude;
mod price;
mod routines;
mod sheets;

use crate::prelude::*;

#[tokio::main]
async fn main() {
    routines::UpdateAirdropWalletBalanceRoutine.run().await
}
