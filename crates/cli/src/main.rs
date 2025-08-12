use crypto_balance_core::prettyprint::prettyprint::PrettyFormatter;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::Resource;
use std::env;
use std::sync::Arc;
use tracing::{error, info, instrument};
use tracing_indicatif::IndicatifLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

mod application_service_factory;
mod cli_adapter;

use application_service_factory::ApplicationServiceFactory;
use cli_adapter::CliAdapter;

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing()?;
    setup_panic_hook();

    let args: Vec<String> = env::args().collect();

    info!("Starting crypto-balance CLI");

    let app_service = ApplicationServiceFactory::create().await?;
    let cli_adapter = Arc::new(CliAdapter::new(app_service));

    match cli_adapter.run(args).await {
        Ok(_) => {
            info!("CLI execution completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("CLI execution failed: {:?}", e);
            Err(e)
        }
    }
}

fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let indicatif_layer = IndicatifLayer::new();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .event_format(PrettyFormatter::new(true))
        .with_writer(indicatif_layer.get_stderr_writer());

    let log_file_layer = tracing_subscriber::fmt::layer()
        .event_format(PrettyFormatter::new(false))
        .with_writer(std::fs::File::create("crypto_balance.log")?)
        .with_ansi(false);

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("http://localhost:4317");

    let tracer =
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "crypto_balance_cli"),
            ])))
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let otel_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("crypto_balance_cli", tracing::Level::TRACE)
                .with_target("crypto_balance_core", tracing::Level::TRACE),
        )
        .with(otel_layer)
        .with(indicatif_layer)
        .with(log_file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}

fn setup_panic_hook() {
    tracing::trace!("Setting panic hook");
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("panic: {info}");
        opentelemetry::global::shutdown_tracer_provider();
    }));
}
