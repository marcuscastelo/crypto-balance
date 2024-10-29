#![feature(lazy_cell)]
#![feature(async_closure)]

mod blockchain;
mod config;
mod exchange;
mod prelude;
mod price;
mod routines;
mod scraping;
mod sheets;

use tokio::process::Command;

use crate::prelude::*;

async fn run_routines(parallel: bool) {
    let _ = Command::new("pkill").arg("geckodriver").output().await;

    let routines_to_run: Vec<&dyn Routine<()>> = vec![
        &routines::toplevel::debank_routine::DebankRoutine,
        // &routines::toplevel::token_prices::TokenPricesRoutine,
        // &routines::toplevel::UpdateBinanceBalanceOnSheetsRoutine,
        // &routines::toplevel::UpdateBybitBalanceOnSheetsRoutine,
        // &routines::toplevel::UpdateKrakenBalanceOnSheetsRoutine,
        // &routines::toplevel::UpdateHoldBalanceOnSheetsRoutine,
    ];

    let mut futures = Vec::new();

    for routine in routines_to_run {
        if parallel {
            futures.push(tokio::spawn(routine.run()));
        } else {
            routine.run().await;
        }
    }

    if parallel {
        futures::future::join_all(futures).await;
    }

    // Kill all geckodriver processes
    // TODO: make this more robust, e.g. by killing only the geckodriver processes that were spawned by this program after each routine
    // TODO: kill processes even if the program panics
    let _ = Command::new("pkill").arg("geckodriver").output();
}

#[tokio::main]
async fn main() {
    env_logger::builder().init();

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = true;
    run_routines(parallel).await;
}
