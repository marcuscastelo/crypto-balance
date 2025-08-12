use crate::ports::routine::RoutineError;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationServiceError {
    #[error("Service initialization failed: {details}")]
    InitializationFailed { details: String },
    #[error("Routine execution failed: {details}")]
    RoutineExecutionFailed { details: String },
    #[error("Multiple routines failed")]
    MultipleFailures {
        failures: HashMap<String, RoutineError>,
    },
}

#[async_trait::async_trait]
pub trait ApplicationService: Send + Sync {
    async fn run_all_routines(
        &self,
        parallel: bool,
    ) -> error_stack::Result<
        HashMap<String, error_stack::Result<(), RoutineError>>,
        ApplicationServiceError,
    >;

    async fn run_routine_by_name(
        &self,
        name: &str,
    ) -> error_stack::Result<(), ApplicationServiceError>;

    async fn list_available_routines(&self) -> Vec<String>;

    async fn health_check(&self) -> error_stack::Result<String, ApplicationServiceError>;
}
