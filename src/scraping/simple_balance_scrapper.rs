use std::process::Command;

use fantoccini::{error::NewSessionError, ClientBuilder, Locator};
use indicatif::ProgressBar;

use crate::cli::progress::{finish_progress, new_progress, ProgressBarExt};

pub struct SimpleBalanceScrapper {
    pub url: String,
    pub xpath: String,
    pub wait_time_secs: u64,
}

impl SimpleBalanceScrapper {
    pub async fn scrape(&self) -> anyhow::Result<f64> {
        // Generate random port every time (above 1024 to avoid permission issues)
        let port = rand::random::<u16>() % (65535 - 1024) + 1024;

        log::trace!("Spawning geckodriver process with port {}", port);
        let mut driver_process = Command::new("geckodriver")
            .arg("--port")
            .arg(port.to_string())
            .arg("--log")
            .arg("fatal")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to start geckodriver");

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        log::trace!("Connecting to WebDriver at port {}", port);
        let c = ClientBuilder::native()
            .connect(format!("http://localhost:{}", port).as_str())
            .await
            .map_err(|error: NewSessionError| {
                anyhow::anyhow!(format!(
                    "Failed to connect to WebDriver: {}",
                    error.to_string()
                ))
            })?;

        log::trace!("Navigating to {}", self.url);
        c.goto(self.url.as_str()).await?;
        let url = c.current_url().await?;

        while url.as_ref() != self.url.as_str() {
            log::warn!("Page loading is taking too long, waiting 1 second...");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        log::trace!("Page loaded successfully");

        let progress = new_progress(ProgressBar::new(self.wait_time_secs));
        let message = format!("Waiting {} seconds for page to load", self.wait_time_secs);
        progress.trace(message);
        for _ in 0..self.wait_time_secs {
            progress.inc(1);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        finish_progress(&progress);

        log::trace!("Finding element with xpath {}", self.xpath);
        let element = c.find(Locator::XPath(self.xpath.as_str())).await?;

        log::trace!("Getting text from element");
        let balance = element.text().await?;

        // Get number between $ and \n (e.g. $1,234.56\n -> 1234.56)
        let balance = balance
            .split('\n')
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to split balance on newline"))?
            .split('$')
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Failed to split balance on dollar sign"))?
            .replace(',', "");

        let balance = balance
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!(format!("Failed to parse balance: {:?}", balance)))?;

        c.close().await?;
        driver_process.kill().expect("Failed to kill geckodriver");

        log::trace!("Returning balance: {}", balance);
        Ok(balance)
    }
}
