use error_stack::ResultExt;
use std::fmt;
use std::process::Child;
use tracing::instrument;

use fantoccini::{Client, ClientBuilder};

use super::scraper_driver::{random_port, spawn_geckodriver_process, ScraperDriverError};

pub struct FantocciniScraperDriver {
    driver_process: Option<Child>,
    pub client: Client,
}

impl fmt::Debug for FantocciniScraperDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FantocciniScraperDriver").finish()
    }
}

#[instrument]
async fn create_and_configure_client(port: u16) -> error_stack::Result<Client, ScraperDriverError> {
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let client = ClientBuilder::native()
        .connect(format!("http://localhost:{}", port).as_str())
        .await
        .change_context(ScraperDriverError::FailedToCreateClient)
        .attach_printable_lazy(|| format!("Failed to connect to geckodriver on port {}", port))?;

    client.set_ua("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 OPR/112.0.0.0").await.change_context(ScraperDriverError::FailedToCreateClient)?;

    Ok(client)
}

impl FantocciniScraperDriver {
    #[instrument]
    pub async fn new() -> error_stack::Result<Self, ScraperDriverError> {
        let port = random_port();

        let scraper = FantocciniScraperDriver {
            driver_process: spawn_geckodriver_process(port).await?.into(),
            client: create_and_configure_client(port).await?.into(),
        };

        Ok(scraper)
    }

    #[instrument]
    pub fn close(&mut self) {
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

impl Drop for FantocciniScraperDriver {
    fn drop(&mut self) {
        self.close();

        // Sleep for a bit to allow future to run (hacky, but it works for now)
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
