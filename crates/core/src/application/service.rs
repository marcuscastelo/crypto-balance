use crate::ports::application_service::{ApplicationService, ApplicationServiceError};
use crate::ports::routine::{Routine, RoutineError};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{instrument, info, error, Instrument};
use futures::future::join_all;

pub struct CryptoBalanceApplicationService {
    routines: Vec<Box<dyn Routine>>,
}

impl CryptoBalanceApplicationService {
    pub fn new(routines: Vec<Box<dyn Routine>>) -> Self {
        Self { routines }
    }
}

#[async_trait::async_trait]
impl ApplicationService for CryptoBalanceApplicationService {
    #[instrument(skip(self))]
    async fn run_all_routines(&self, parallel: bool) -> error_stack::Result<HashMap<String, error_stack::Result<(), RoutineError>>, ApplicationServiceError> {
        let mut routine_results: HashMap<String, error_stack::Result<(), RoutineError>> = HashMap::new();

        if parallel {
            info!("Running {} routines in parallel", self.routines.len());
            
            let futures: Vec<_> = self.routines.iter().enumerate().map(|(index, routine)| {
                routine.run().instrument(tracing::span!(
                    tracing::Level::INFO,
                    "routine",
                    routine = routine.name(),
                    index = index,
                    len = self.routines.len()
                ))
            }).collect();

            let results = join_all(futures).await;
            
            for (routine, result) in self.routines.iter().zip(results) {
                let name = routine.name().to_string();
                routine_results.insert(name, result);
            }
        } else {
            info!("Running {} routines sequentially", self.routines.len());
            
            for (index, routine) in self.routines.iter().enumerate() {
                let result = routine
                    .run()
                    .instrument(tracing::span!(
                        tracing::Level::INFO,
                        "routine",
                        routine = routine.name(),
                        index = index,
                        len = self.routines.len()
                    ))
                    .await;

                if let Err(ref report) = result {
                    error!("❌ {}: {:?}", routine.name(), report);
                } else {
                    info!("✅ {}: OK", routine.name());
                }
                
                routine_results.insert(routine.name().to_string(), result);
            }
        }

        Ok(routine_results)
    }

    #[instrument(skip(self))]
    async fn run_routine_by_name(&self, name: &str) -> error_stack::Result<(), ApplicationServiceError> {
        let routine = self.routines
            .iter()
            .find(|r| r.name() == name)
            .ok_or_else(|| ApplicationServiceError::RoutineExecutionFailed {
                details: format!("Routine '{}' not found", name),
            })?;

        routine
            .run()
            .await
            .map_err(|e| ApplicationServiceError::RoutineExecutionFailed {
                details: format!("Routine '{}' failed: {:?}", name, e),
            })?;

        Ok(())
    }

    async fn list_available_routines(&self) -> Vec<String> {
        self.routines.iter().map(|r| r.name().to_string()).collect()
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> error_stack::Result<String, ApplicationServiceError> {
        let routine_count = self.routines.len();
        let routine_names: Vec<String> = self.list_available_routines().await;
        
        Ok(format!(
            "🟢 Crypto Balance Service - Healthy\n\
             Routines available: {}\n\
             Routine names: {}",
            routine_count,
            routine_names.join(", ")
        ))
    }
}