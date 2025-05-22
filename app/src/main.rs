#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

mod blockchain;
mod config;
mod exchange;
mod prelude;
mod price;
mod routines;
mod scraping;
mod sheets;

use exchange::data::{
    binance::binance_use_cases::BinanceUseCases, kraken::kraken_use_cases::KrakenUseCases,
};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::Resource;
use routines::{
    debank_tokens_routine::DebankTokensRoutine,
    debank_total_usd_routine::DebankTotalUSDRoutine,
    exchange_balances_routine::ExchangeBalancesRoutine,
    routine::{Routine, RoutineFailureInfo, RoutineResult},
    token_prices::TokenPricesRoutine,
};
use std::{collections::HashMap, fs::File};
use tokio::{process::Command, time::sleep, time::Duration};
use tracing::instrument;
use tracing_chrome::ChromeLayerBuilder;
use tracing_flame::FlameLayer;
use tracing_indicatif::IndicatifLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{self};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry};

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

    for (index, routine) in routines_to_run.iter().enumerate() {
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
            if index < routines_to_run.len() - 1 {
                let span = tracing::span!(
                    tracing::Level::INFO,
                    "wait",
                    last_routine = routine.name(),
                    index = index,
                    len = routines_to_run.len()
                );
                let _enter = span.enter();
                let secs = 15;
                tracing::info!(
                    "Waiting for {} seconds before running the next routine...",
                    secs
                );
                sleep(Duration::from_secs(secs)).await;
            }
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
    let indicatif_layer = IndicatifLayer::new();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_ansi(true)
        .with_target(false)
        .with_line_number(true)
        .with_file(true);

    // let file = File::create("log.ndjson").unwrap();
    // let json_layer = tracing_subscriber::fmt::layer()
    //     .json()
    //     .with_writer(file)
    //     .with_span_events(fmt::format::FmtSpan::FULL);

    // let (chrome_layer, _guard) = ChromeLayerBuilder::new()
    //     .file("chrome_trace.json") // nome do arquivo final
    //     .include_args(true)
    //     .build();

    // let file = File::create("flame.folded").unwrap();
    // let flame_layer = FlameLayer::new(file);

    // Cria um tracer que envia para o agente Jaeger local
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic() // para gRPC
        .with_endpoint("http://localhost:4317"); // default do OTel Collector ou Tempo

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
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(stdout_layer)
        // .with(json_layer)
        // .with(chrome_layer)
        // .with(flame_layer)
        .with(otel_layer)
        .with(indicatif_layer)
        .init();

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = false;
    run_routines(parallel).await;

    opentelemetry::global::shutdown_tracer_provider();
}
