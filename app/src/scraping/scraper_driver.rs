use error_stack::{Context, Result, ResultExt};
use std::fmt;
use std::process::{Child, Command};

use fantoccini::{error::NewSessionError, Client, ClientBuilder};

#[derive(Debug)]
pub struct ScraperDriver {
    driver_process: Option<Child>,
    pub client: Client,
}

#[derive(Debug)]
pub enum ScraperDriverError {
    FailedToSpawnGeckodriver,
    FailedToCreateClient,
}

impl Context for ScraperDriverError {}

impl fmt::Display for ScraperDriverError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

fn random_port() -> u16 {
    rand::random::<u16>() % (65535 - 1024) + 1024
}

async fn spawn_geckodriver_process(port: u16) -> Result<Child, ScraperDriverError> {
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

async fn create_and_configure_client(port: u16) -> Result<Client, ScraperDriverError> {
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let client = ClientBuilder::native()
        .connect(format!("http://localhost:{}", port).as_str())
        .await
        .change_context(ScraperDriverError::FailedToCreateClient)?;

    client.set_ua("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 OPR/112.0.0.0").await.change_context(ScraperDriverError::FailedToCreateClient)?;

    Ok(client)
}

impl ScraperDriver {
    pub async fn new() -> Result<Self, ScraperDriverError> {
        let port = random_port();

        let scraper = ScraperDriver {
            driver_process: spawn_geckodriver_process(port).await?.into(),
            client: create_and_configure_client(port).await?.into(),
        };

        Ok(scraper)
    }

    pub fn close(&mut self) {
        tracing::info!("Closing ScraperDriver");
        let process = self
            .driver_process
            .take()
            .ok_or_else(|| anyhow::anyhow!("No geckodriver process to close"));

        let client_clone = self.client.clone();
        let client = std::mem::replace(&mut self.client, client_clone);

        let future = async {
            client.close().await.unwrap_or_else(|error| {
                tracing::error!("Failed to close WebDriver client: {}", error)
            });

            if let Ok(mut process) = process {
                process.kill().unwrap_or_else(|error| {
                    tracing::error!("Failed to kill geckodriver process: {}", error)
                })
            } else {
                tracing::error!("Failed to close geckodriver process")
            }
        };

        tokio::spawn(future);
    }
}

impl Drop for ScraperDriver {
    fn drop(&mut self) {
        self.close();

        // Sleep for a bit to allow future to run (hacky, but it works for now)
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
