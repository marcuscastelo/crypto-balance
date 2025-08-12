use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {details}")]
    InvalidCommand { details: String },
    #[error("Command execution failed: {details}")]
    ExecutionFailed { details: String },
}

#[derive(Debug, Clone)]
pub enum Command {
    RunRoutines { parallel: bool },
    RunSpecificRoutine { name: String },
    ListRoutines,
    HealthCheck,
}

#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, command: Command) -> error_stack::Result<String, CommandError>;
}