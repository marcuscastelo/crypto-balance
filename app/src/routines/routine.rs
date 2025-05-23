use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoutineError {
    #[error("Routine failed: {0}")]
    RoutineFailure(String),
}

#[async_trait::async_trait]
pub trait Routine {
    fn name(&self) -> &str;

    async fn run(&self) -> error_stack::Result<(), RoutineError>;
}
