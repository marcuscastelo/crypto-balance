use std::process::{Child, Command};

use fantoccini::{error::NewSessionError, Client, ClientBuilder};

pub struct ScraperDriver {
    driver_process: Option<Child>,
    pub client: Client,
}

fn random_port() -> u16 {
    rand::random::<u16>() % (65535 - 1024) + 1024
}

async fn spawn_geckodriver_process(port: u16) -> anyhow::Result<Child> {
    Command::new("geckodriver")
        .arg("--port")
        .arg(port.to_string())
        .arg("--log")
        .arg("fatal")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| anyhow::anyhow!(format!("Failed to start geckodriver: {}", error)))
}

async fn create_and_configure_client(port: u16) -> anyhow::Result<Client> {
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let client = ClientBuilder::native()
        .connect(format!("http://localhost:{}", port).as_str())
        .await
        .map_err(|error: NewSessionError| {
            anyhow::anyhow!(format!(
                "Failed to connect to WebDriver: {}",
                error.to_string()
            ))
        })?;

    client.set_ua("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 OPR/112.0.0.0").await?;

    Ok(client)
}

impl ScraperDriver {
    pub async fn new() -> anyhow::Result<Self> {
        let port = random_port();

        let scraper = ScraperDriver {
            driver_process: spawn_geckodriver_process(port).await?.into(),
            client: create_and_configure_client(port).await?.into(),
        };

        Ok(scraper)
    }
}

impl Drop for ScraperDriver {
    fn drop(&mut self) {
        let process = self
            .driver_process
            .take()
            .ok_or_else(|| log::error!("Driver process not found"));

        if let Ok(mut process) = process {
            process.kill().unwrap_or_else(|error| {
                log::error!("Failed to kill geckodriver process: {}", error)
            })
        }
    }
}
