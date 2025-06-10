use std::process::{Child, Command};

use error_stack::ResultExt;
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error)]
pub enum ScraperDriverError {
    #[error("Failed to spawn geckodriver process")]
    FailedToSpawnGeckodriver,
    #[error("Failed to create client for geckodriver")]
    FailedToCreateClient,
}

pub fn random_port() -> u16 {
    rand::random::<u16>() % (65535 - 1024) + 1024
}

#[instrument]
pub async fn spawn_geckodriver_process(
    port: u16,
) -> error_stack::Result<Child, ScraperDriverError> {
    Command::new("geckodriver")
        .arg("--port")
        .arg(port.to_string())
        .arg("--log")
        .arg("fatal")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .change_context(ScraperDriverError::FailedToSpawnGeckodriver)
}
