use crypto_balance_core::ports::application_service::ApplicationService;
use crypto_balance_core::ports::event_handler::{CryptoEvent, EventError, EventHandler};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, DefaultConsumerContext};
use rdkafka::message::Message;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{error, info, instrument, warn};

pub struct KafkaEventAdapter {
    consumer: StreamConsumer<DefaultConsumerContext>,
    application_service: Arc<dyn ApplicationService>,
    topics: Vec<String>,
}

impl KafkaEventAdapter {
    pub fn new(
        brokers: &str,
        group_id: &str,
        topics: Vec<String>,
        application_service: Arc<dyn ApplicationService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("auto.offset.reset", "latest")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()?;

        consumer.subscribe(&topics.iter().map(String::as_str).collect::<Vec<_>>())?;

        Ok(Self {
            consumer,
            application_service,
            topics,
        })
    }

    #[instrument(skip(self))]
    pub async fn start_consuming(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Kafka consumer for topics: {:?}", self.topics);

        loop {
            match timeout(Duration::from_millis(100), self.consumer.recv()).await {
                Ok(Ok(message)) => {
                    let payload = match message.payload_view::<str>() {
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            error!("Error parsing payload: {:?}", e);
                            continue;
                        }
                        None => {
                            warn!("Empty payload received");
                            continue;
                        }
                    };

                    info!(
                        "Received message from topic: {}, partition: {}, offset: {}",
                        message.topic(),
                        message.partition(),
                        message.offset()
                    );

                    match self.process_message(payload).await {
                        Ok(_) => {
                            if let Err(e) = self.consumer.commit_message(&message, CommitMode::Async) {
                                error!("Failed to commit message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to process message: {:?}", e);
                            // For now, we'll commit to avoid infinite retries
                            if let Err(commit_err) = self.consumer.commit_message(&message, CommitMode::Async) {
                                error!("Failed to commit failed message: {:?}", commit_err);
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Kafka consumer error: {:?}", e);
                }
                Err(_) => {
                    // Timeout - this is normal, continue listening
                    continue;
                }
            }
        }
    }

    #[instrument(skip(self, payload))]
    async fn process_message(&self, payload: &str) -> error_stack::Result<(), EventError> {
        let event: CryptoEvent = serde_json::from_str(payload)
            .map_err(|e| error_stack::report!(EventError::InvalidEvent {
                details: format!("Failed to deserialize event: {}", e),
            }))?;

        info!("Processing event: {:?}", event);
        self.handle(event).await
    }
}

#[async_trait::async_trait]
impl EventHandler for KafkaEventAdapter {
    #[instrument(skip(self))]
    async fn handle(&self, event: CryptoEvent) -> error_stack::Result<(), EventError> {
        match event {
            CryptoEvent::RunBalanceUpdate { exchange, .. } => {
                // Try to find routine name based on exchange
                let routine_name = match exchange.as_str() {
                    "Binance" => "BinanceBalancesRoutine",
                    "Kraken" => "KrakenBalancesRoutine", 
                    _ => {
                        return Err(error_stack::report!(EventError::ProcessingFailed {
                            details: format!("Unknown exchange: {}", exchange),
                        }));
                    }
                };

                self.application_service
                    .run_routine_by_name(routine_name)
                    .await
                    .map_err(|e| error_stack::report!(EventError::ProcessingFailed {
                        details: format!("Failed to run balance update for {}: {:?}", exchange, e),
                    }))?;
            }
            CryptoEvent::RunPriceUpdate { .. } => {
                self.application_service
                    .run_routine_by_name("TokenPricesRoutine")
                    .await
                    .map_err(|e| error_stack::report!(EventError::ProcessingFailed {
                        details: format!("Failed to run price update: {:?}", e),
                    }))?;
            }
            CryptoEvent::RunDebankUpdate { .. } => {
                self.application_service
                    .run_routine_by_name("DebankRoutine")
                    .await
                    .map_err(|e| error_stack::report!(EventError::ProcessingFailed {
                        details: format!("Failed to run debank update: {:?}", e),
                    }))?;
            }
            CryptoEvent::HealthCheck { .. } => {
                let health = self.application_service
                    .health_check()
                    .await
                    .map_err(|e| error_stack::report!(EventError::ProcessingFailed {
                        details: format!("Health check failed: {:?}", e),
                    }))?;
                info!("Health check result: {}", health);
            }
        }

        Ok(())
    }
}