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
mod sheets;

use cli::progress::CLI_MULTI_PROGRESS;
use exchange::data::{
    binance::binance_use_cases::BinanceUseCases, kraken::kraken_use_cases::KrakenUseCases,
};
use indicatif_log_bridge::LogWrapper;
use routines::{
    debank_tokens_routine::DebankTokensRoutine,
    debank_total_usd_routine::DebankTotalUSDRoutine,
    exchange_balances_routine::ExchangeBalancesRoutine,
    routine::{Routine, RoutineFailureInfo, RoutineResult},
    token_prices::TokenPricesRoutine,
    update_hold_balance_on_sheets::UpdateHoldBalanceOnSheetsRoutine,
};
use std::collections::HashMap;
use tokio::{process::Command, time::sleep, time::Duration};
use tracing::instrument;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use tracing_subscriber::{self};

#[instrument]
async fn run_routines(parallel: bool) {
    let _ = Command::new("pkill").arg("geckodriver").output().await;

    let routines_to_run: Vec<Box<dyn Routine>> = vec![
        Box::new(DebankTokensRoutine),
        Box::new(DebankTotalUSDRoutine),
        Box::new(TokenPricesRoutine),
        Box::new(ExchangeBalancesRoutine::new(&BinanceUseCases)),
        Box::new(ExchangeBalancesRoutine::new(&KrakenUseCases)),
        // Box::new(SonarWatchRoutine),
        // Box::new(UpdateHoldBalanceOnSheetsRoutine),
    ];

    let mut futures = Vec::new();

    let mut routine_results: HashMap<String, RoutineResult> = HashMap::new();

    for routine in routines_to_run.iter() {
        if parallel {
            futures.push(routine.run());
        } else {
            let result = routine.run().await;
            if let Err(err) = &result {
                tracing::error!("❌ {}: {}", routine.name(), err.message);
            } else {
                tracing::info!("✅ {}: OK", routine.name());
            }
            routine_results.insert(routine.name().to_string(), result);
            tracing::info!("Waiting for 30 seconds before running the next routine...");
            sleep(Duration::from_secs(30)).await;
        }
    }

    if parallel {
        let future_results = futures::future::join_all(futures).await;
        for (routine, result) in routines_to_run.iter().zip(future_results) {
            routine_results.insert(routine.name().to_string(), result);
        }
    }

    tracing::info!("Routine results:");
    for (name, result) in routine_results {
        match result {
            Ok(()) => {
                tracing::info!("✅ {}: OK", name);
            }
            Err(failure_info) => {
                tracing::error!("❌ {}: {}", name, failure_info.message);
            }
        }
    }

    // Kill all geckodriver processes
    // TODO: make this more robust, e.g. by killing only the geckodriver processes that were spawned by this program after each routine
    // TODO: kill processes even if the program panics
    let _ = Command::new("pkill").arg("geckodriver").output();
}

#[tokio::main]
#[instrument]
async fn main() {
    // let logger = env_logger::builder().build(); // TODO: remove env_logger dependency
    // let level = logger.filter();
    // LogWrapper::new(CLI_MULTI_PROGRESS.clone(), logger)
    //     .try_init()
    //     .expect("Failed to initialize logger");
    // tracing::set_max_level(level);
    let indicatif_layer = IndicatifLayer::new();
    let subscriber = Registry::default()
        .with(tracing_subscriber::filter::LevelFilter::WARN)
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = false;
    run_routines(parallel).await;

    // CLI_MULTI_PROGRESS.clear().unwrap();
}
