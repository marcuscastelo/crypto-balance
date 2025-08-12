use crate::ports::event_handler::{CryptoEvent, EventError, EventPublisher};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tracing::{error, info, instrument};

pub struct KafkaEventPublisher {
    producer: FutureProducer,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.messages", "10000")
            .set("queue.buffering.max.ms", "1000")
            .set("batch.size", "16384")
            .create()?;

        Ok(Self { producer })
    }
}

#[async_trait::async_trait]
impl EventPublisher for KafkaEventPublisher {
    #[instrument(skip(self, event))]
    async fn publish(
        &self,
        topic: &str,
        event: CryptoEvent,
    ) -> error_stack::Result<(), EventError> {
        let payload = serde_json::to_string(&event).map_err(|e| EventError::RoutingFailed {
            details: format!("Failed to serialize event: {}", e),
        })?;

        let key = format!("crypto-balance-{}", chrono::Utc::now().timestamp());
        let record = FutureRecord::to(topic).key(&key).payload(&payload);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok((partition, offset)) => {
                info!(
                    "Event published successfully to topic: {}, partition: {}, offset: {}",
                    topic, partition, offset
                );
                Ok(())
            }
            Err((e, _)) => {
                error!("Failed to publish event to topic {}: {:?}", topic, e);
                Err(error_stack::report!(EventError::RoutingFailed {
                    details: format!("Failed to publish to Kafka: {:?}", e),
                }))
            }
        }
    }
}
