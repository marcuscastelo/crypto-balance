use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::Resource;
use std::env;
use tokio::signal;
use tracing::{error, info, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

mod kafka_adapter;
mod application_service_factory;

use kafka_adapter::KafkaEventAdapter;
use application_service_factory::ApplicationServiceFactory;

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing()?;
    setup_panic_hook();

    info!("Starting crypto-balance Kafka consumer");

    let brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let group_id = env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "crypto-balance-group".to_string());
    let topics = env::var("KAFKA_TOPICS")
        .unwrap_or_else(|_| "crypto-balance-events".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    info!(
        "Kafka config - Brokers: {}, Group ID: {}, Topics: {:?}",
        brokers, group_id, topics
    );

    let app_service = ApplicationServiceFactory::create().await?;
    let kafka_adapter = KafkaEventAdapter::new(&brokers, &group_id, topics, app_service)?;

    // Setup graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl-c");
        info!("Received shutdown signal, stopping Kafka consumer...");
    };

    tokio::select! {
        result = kafka_adapter.start_consuming() => {
            match result {
                Ok(_) => {
                    info!("Kafka consumer finished normally");
                    Ok(())
                }
                Err(e) => {
                    error!("Kafka consumer error: {:?}", e);
                    Err(e)
                }
            }
        }
        _ = shutdown_signal => {
            info!("Graceful shutdown completed");
            Ok(())
        }
    }
}

fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout);

    let log_file_layer = tracing_subscriber::fmt::layer()
        .with_writer(
            std::fs::File::create("crypto_balance_kafka.log")?,
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
                KeyValue::new("service.name", "crypto_balance_kafka"),
            ])))
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let otel_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("crypto_balance_kafka", tracing::Level::TRACE)
                .with_target("crypto_balance_core", tracing::Level::TRACE),
        )
        .with(otel_layer)
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