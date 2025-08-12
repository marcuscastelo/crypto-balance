use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutineError {
    #[error("Routine failed: {details}")]
    RoutineFailure { details: String },
}

impl RoutineError {
    pub fn routine_failure<S: Into<String>>(details: S) -> Self {
        RoutineError::RoutineFailure {
            details: details.into(),
        }
    }
}

#[async_trait::async_trait]
pub trait Routine: Send + Sync {
    fn name(&self) -> &str;

    async fn run(&self) -> error_stack::Result<(), RoutineError>;
}