#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

mod application;
mod domain;
mod infrastructure;
mod prettyprint;

use application::debank::debank_routine::DebankRoutine;
use application::exchange::binance_use_cases::BinanceUseCases;
use application::exchange::exchange_balances_routine::ExchangeBalancesRoutine;
use application::exchange::kraken_use_cases::KrakenUseCases;

use application::price::token_prices::TokenPricesRoutine;
use domain::exchange::BalanceRepository;
use domain::routine::Routine;
use domain::routine::RoutineError;
use infrastructure::config::app_config::CONFIG;
use infrastructure::exchange::binance_factory::BinanceAccountFactory;
use infrastructure::exchange::kraken_factory::KrakenFactory;
use infrastructure::exchange::spreadsheet_balance_repository::SpreadsheetBalanceRepository;
// External
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::Resource;
use prettyprint::prettyprint::PrettyFormatter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tracing::instrument;
use tracing::Instrument;
use tracing_indicatif::IndicatifLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{self};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

#[instrument]
async fn run_routines(parallel: bool) {
    let _ = Command::new("pkill").arg("geckodriver").output().await;
    let spreadsheet_manager = Arc::new(
        infrastructure::sheets::spreadsheet_manager::SpreadsheetManager::new(CONFIG.sheets.clone())
            .await,
    );

    let balance_repository: Arc<dyn BalanceRepository> = Arc::new(
        SpreadsheetBalanceRepository::new(Arc::clone(&spreadsheet_manager)),
    );

    let routines_to_run: Vec<Box<dyn Routine>> = vec![
        Box::new(DebankRoutine::new(
            CONFIG.blockchain.airdrops.evm.clone(),
            Arc::clone(&spreadsheet_manager),
        )),
        Box::new(TokenPricesRoutine::new(&spreadsheet_manager)),
        Box::new(ExchangeBalancesRoutine::new(
            BinanceUseCases::new(BinanceAccountFactory::new(CONFIG.binance.clone())),
            Arc::clone(&balance_repository),
        )),
        Box::new(ExchangeBalancesRoutine::new(
            KrakenUseCases::new(KrakenFactory::new(CONFIG.kraken.clone())),
            Arc::clone(&balance_repository),
        )),
        // Box::new(UpdateHoldBalanceOnSheetsRoutine),
    ];

    let mut futures = Vec::new();

    let mut routine_results: HashMap<String, error_stack::Result<(), RoutineError>> =
        HashMap::new();

    for (index, routine) in routines_to_run.iter().enumerate() {
        if parallel {
            futures.push(routine.run());
        } else {
            let result = routine
                .run()
                .instrument(tracing::span!(
                    tracing::Level::INFO,
                    "routine",
                    routine = routine.name(),
                    index = index,
                    len = routines_to_run.len()
                ))
                .await;
            if let Err(report) = &result {
                tracing::error!("❌ {}: {:?}", routine.name(), report);
            } else {
                tracing::info!("✅ {}: OK", routine.name());
            }
            routine_results.insert(routine.name().to_string(), result);
            // if index < routines_to_run.len() - 1 {
            //     let span = tracing::span!(
            //         tracing::Level::INFO,
            //         "wait",
            //         last_routine = routine.name(),
            //         index = index,
            //         len = routines_to_run.len()
            //     );
            //     let _enter = span.enter();
            //     let secs = 15;
            //     tracing::info!(
            //         "Waiting for {} seconds before running the next routine...",
            //         secs
            //     );
            //     sleep(Duration::from_secs(secs)).await;
            // }
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
            Err(report) => {
                tracing::error!("❌ {}: {:?}", name, report);
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
    let indicatif_layer = IndicatifLayer::new();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(PrettyFormatter::new(true))
        .with_writer(indicatif_layer.get_stderr_writer());

    let log_file_layer = tracing_subscriber::fmt::layer()
        .event_format(PrettyFormatter::new(false))
        .with_writer(
            std::fs::File::create("crypto_balance.log").expect("Failed to create log file"),
        )
        .with_ansi(false);

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");

    let tracer =
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "crypto_balance"),
            ])))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to install OTLP tracer");

    let otel_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("crypto_balance", tracing::Level::TRACE),
        )
        .with(otel_layer)
        .with(indicatif_layer)
        .with(log_file_layer)
        .with(stdout_layer)
        .init();

    tracing::trace!("Setting panic hook");
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("panic: {info}");
        opentelemetry::global::shutdown_tracer_provider();
    }));

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = true;
    run_routines(parallel).await;

    opentelemetry::global::shutdown_tracer_provider();
}
