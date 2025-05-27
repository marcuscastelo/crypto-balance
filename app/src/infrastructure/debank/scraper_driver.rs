use std::process::{Child, Command};

use error_stack::ResultExt;
use thiserror::Error;
use tracing::instrument;

pub trait ScraperDriver {
    type Selector;
    async fn visit_url(&mut self, url: &str) -> error_stack::Result<(), ScraperDriverError>;
    async fn wait_for_url(&mut self, url: &str) -> error_stack::Result<(), ScraperDriverError>;
    async fn close(&mut self) -> error_stack::Result<(), ScraperDriverError>;
    async fn find(
        &mut self,
        selector: Self::Selector,
    ) -> error_stack::Result<(), ScraperDriverError>;
    async fn find_all(
        &mut self,
        selector: Self::Selector,
    ) -> error_stack::Result<(), ScraperDriverError>;
}

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
