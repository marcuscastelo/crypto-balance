#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

mod blockchain;
mod cli;
mod config;
mod exchange;
mod prelude;
mod price;
mod routines;
mod scraping;
mod script;
mod sheets;

use std::collections::HashMap;

use cli::progress::CLI_MULTI_PROGRESS;
use indicatif_log_bridge::LogWrapper;
use routines::routine::{Routine, RoutineFailureInfo, RoutineResult};
use tokio::process::Command;

use crate::prelude::*;

async fn run_routines(parallel: bool) {
    let _ = Command::new("pkill").arg("geckodriver").output().await;

    let routines_to_run: Vec<&dyn Routine> = vec![
        // &routines::toplevel::debank_routine::DebankRoutine,
        // &routines::toplevel::sonar_watch_routine::SonarWatch,
        &routines::toplevel::token_prices::TokenPricesRoutine,
        &routines::toplevel::binance_routine::BinanceRoutine,
        &routines::toplevel::bybit_routine::BybitRoutine,
        &routines::toplevel::kraken_routine::KrakenRoutine,
        // &routines::toplevel::UpdateHoldBalanceOnSheetsRoutine,
    ];

    let mut futures = Vec::new();

    let mut routine_results: HashMap<String, RoutineResult> = HashMap::new();

    for routine in routines_to_run.iter() {
        if parallel {
            let future = tokio::spawn(routine.run());
            futures.push(future);
        } else {
            let result = routine.run().await;
            routine_results.insert(routine.name().to_string(), result);
        }
    }

    if parallel {
        let future_results = futures::future::join_all(futures).await;
        for (routine, join_result) in routines_to_run.iter().zip(future_results) {
            let routine_result = match join_result {
                Ok(result) => result,
                Err(e) => Err(RoutineFailureInfo::new(e.to_string())),
            };

            routine_results.insert(routine.name().to_string(), routine_result);
        }
    }

    for (name, result) in routine_results {
        match result {
            Ok(()) => {
                log::info!("✅ {}: OK", name);
            }
            Err(failure_info) => {
                log::error!("❌ {}: {}", name, failure_info.message);
            }
        }
    }

    // Kill all geckodriver processes
    // TODO: make this more robust, e.g. by killing only the geckodriver processes that were spawned by this program after each routine
    // TODO: kill processes even if the program panics
    let _ = Command::new("pkill").arg("geckodriver").output();
}

#[tokio::main]
async fn main() {
    let logger = env_logger::builder().build();

    let level = logger.filter();

    LogWrapper::new(CLI_MULTI_PROGRESS.clone(), logger)
        .try_init()
        .expect("Failed to initialize logger");

    log::set_max_level(level);

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = true;
    run_routines(parallel).await;

    CLI_MULTI_PROGRESS.clear().unwrap();
}
