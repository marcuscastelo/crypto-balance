#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

mod application;
mod domain;
mod infrastructure;
mod prettyprint;

use application::debank::debank_routine::DebankRoutine;
use application::exchange::binance::BinanceUseCases;
use application::exchange::exchange_balances_routine::ExchangeBalancesRoutine;
use application::exchange::kraken::KrakenUseCases;

use application::price::token_prices::TokenPricesRoutine;
use application::sheets::spreadsheet::SpreadsheetUseCasesImpl;
use domain::routine::Routine;
use domain::routine::RoutineError;
use infrastructure::config::app_config::CONFIG;
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
    let spreadsheet_manager =
        infrastructure::sheets::spreadsheet_manager::SpreadsheetManager::new(CONFIG.sheets.clone())
            .await;

    let persistence = Arc::new(SpreadsheetUseCasesImpl::new(&spreadsheet_manager));

    let routines_to_run: Vec<Box<dyn Routine>> = vec![
        Box::new(DebankRoutine::new(&spreadsheet_manager)),
        Box::new(TokenPricesRoutine::new(&spreadsheet_manager)),
        Box::new(ExchangeBalancesRoutine::new(
            &BinanceUseCases,
            persistence.clone(),
        )),
        Box::new(ExchangeBalancesRoutine::new(&KrakenUseCases, persistence)),
        // Box::new(SonarWatchRoutine),
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
        .event_format(PrettyFormatter::new())
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_ansi(true);

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
        // .with(json_layer)
        // .with(chrome_layer)
        // .with(flame_layer)
        .with(otel_layer)
        .with(indicatif_layer)
        .with(stdout_layer)
        .init();

    tracing::trace!("Setting panic hook");
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("panic: {info}");

        // tenta forçar exportação
        opentelemetry::global::shutdown_tracer_provider();
    }));

    // TODO: Add a CLI flag to toggle parallelism
    let parallel = true;
    run_routines(parallel).await;

    opentelemetry::global::shutdown_tracer_provider();
}
