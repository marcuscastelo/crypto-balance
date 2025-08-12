use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Invalid event format: {details}")]
    InvalidEvent { details: String },
    #[error("Event processing failed: {details}")]
    ProcessingFailed { details: String },
    #[error("Event routing failed: {details}")]
    RoutingFailed { details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoEvent {
    RunBalanceUpdate {
        exchange: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RunPriceUpdate {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RunDebankUpdate {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    HealthCheck {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: CryptoEvent) -> error_stack::Result<(), EventError>;
}

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, topic: &str, event: CryptoEvent) -> error_stack::Result<(), EventError>;
}