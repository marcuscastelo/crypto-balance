use crypto_balance_core::adapters::kafka_publisher::KafkaEventPublisher;
use crypto_balance_core::ports::application_service::ApplicationService;
use crypto_balance_core::ports::command_handler::{Command, CommandError, CommandHandler};
use crypto_balance_core::ports::event_handler::{CryptoEvent, EventPublisher};
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct CliAdapter {
    application_service: Arc<dyn ApplicationService>,
    kafka_publisher: Option<Arc<KafkaEventPublisher>>,
}

impl std::fmt::Debug for CliAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliAdapter")
            .field("application_service", &"<ApplicationService>")
            .finish()
    }
}

impl CliAdapter {
    pub fn new(application_service: Arc<dyn ApplicationService>) -> Self {
        // Tenta criar o publisher Kafka lendo brokers do env ou default
        let brokers =
            std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let kafka_publisher = match KafkaEventPublisher::new(&brokers) {
            Ok(p) => Some(Arc::new(p)),
            Err(e) => {
                tracing::warn!("KafkaEventPublisher não inicializado: {e}");
                None
            }
        };
        Self {
            application_service,
            kafka_publisher,
        }
    }

    #[instrument]
    pub async fn run(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let command = self.parse_args(args)?;

        // Se for rotina Debank, publica evento Kafka antes de executar
        if let Command::RunRoutines { .. } | Command::RunSpecificRoutine { .. } = &command {
            if let Some(publisher) = &self.kafka_publisher {
                let event = CryptoEvent::RunDebankUpdate {
                    timestamp: chrono::Utc::now(),
                };
                // Tópico padrão
                let topic = "user.sync";
                match publisher.publish(topic, event).await {
                    Ok(_) => info!("Evento user.sync.request publicado no Kafka"),
                    Err(e) => error!("Falha ao publicar evento Kafka: {e:?}"),
                }
            }
        }

        match self.handle(command).await {
            Ok(result) => {
                info!("{}", result);
                Ok(())
            }
            Err(report) => {
                error!("Command failed: {:?}", report);
                Err(format!("Command failed: {:?}", report).into())
            }
        }
    }

    fn parse_args(&self, args: Vec<String>) -> Result<Command, CommandError> {
        match args.get(1).map(|s| s.as_str()) {
            Some("run") => {
                let parallel = !args.iter().any(|arg| arg == "--sequential");
                Ok(Command::RunRoutines { parallel })
            }
            Some("run-routine") => {
                let name = args
                    .get(2)
                    .ok_or_else(|| CommandError::InvalidCommand {
                        details: "Routine name required".to_string(),
                    })?
                    .clone();
                Ok(Command::RunSpecificRoutine { name })
            }
            Some("list") => Ok(Command::ListRoutines),
            Some("health") => Ok(Command::HealthCheck),
            _ => Ok(Command::RunRoutines { parallel: true }), // Default behavior
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler for CliAdapter {
    #[instrument]
    async fn handle(&self, command: Command) -> error_stack::Result<String, CommandError> {
        match command {
            Command::RunRoutines { parallel } => {
                let results = self
                    .application_service
                    .run_all_routines(parallel)
                    .await
                    .map_err(|e| CommandError::ExecutionFailed {
                        details: format!("Failed to run routines: {:?}", e),
                    })?;

                let mut success_count = 0;
                let mut failure_count = 0;
                let mut output = "\nRoutine Results:\n".to_string();

                for (name, result) in results {
                    match result {
                        Ok(()) => {
                            success_count += 1;
                            output.push_str(&format!("✅ {}: OK\n", name));
                        }
                        Err(error) => {
                            failure_count += 1;
                            output.push_str(&format!("❌ {}: {:?}\n", name, error));
                        }
                    }
                }

                output.push_str(&format!(
                    "\nSummary: {} successful, {} failed",
                    success_count, failure_count
                ));

                Ok(output)
            }
            Command::RunSpecificRoutine { name } => {
                self.application_service
                    .run_routine_by_name(&name)
                    .await
                    .map_err(|e| CommandError::ExecutionFailed {
                        details: format!("Failed to run routine {}: {:?}", name, e),
                    })?;

                Ok(format!("✅ Routine '{}' completed successfully", name))
            }
            Command::ListRoutines => {
                let routines = self.application_service.list_available_routines().await;
                Ok(format!("Available routines:\n{}", routines.join("\n")))
            }
            Command::HealthCheck => {
                let health = self.application_service.health_check().await.map_err(|e| {
                    CommandError::ExecutionFailed {
                        details: format!("Health check failed: {:?}", e),
                    }
                })?;

                Ok(health)
            }
        }
    }
}
