use std::process::Command;

use fantoccini::{ClientBuilder, Locator};

pub struct SimpleBalanceScrapper {
    pub url: String,
    pub xpath: String,
    pub wait_time: u64,
}

impl SimpleBalanceScrapper {
    pub async fn scrape(&self) -> anyhow::Result<f64> {
        // Generate random port every time (above 1024 to avoid permission issues)
        let port = rand::random::<u16>() % (65535 - 1024) + 1024;

        let mut driver_process = Command::new("geckodriver")
            .arg("--port")
            .arg(port.to_string())
            .arg("--log")
            .arg("fatal")
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("Failed to start geckodriver");

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let c = ClientBuilder::native()
            .connect(format!("http://localhost:{}", port).as_str())
            .await
            .expect("failed to connect to WebDriver");

        c.goto(self.url.as_str()).await?;
        let url = c.current_url().await?;

        while url.as_ref() != self.url.as_str() {
            log::info!("Waiting for url to match");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(self.wait_time)).await;

        let element = c.find(Locator::XPath(self.xpath.as_str())).await?;

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

        Ok(balance)
    }
}
