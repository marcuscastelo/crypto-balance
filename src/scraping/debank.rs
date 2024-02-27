use std::process::Command;

use fantoccini::{ClientBuilder, Locator};

pub struct DebankScraper;

impl DebankScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        let mut driver_process = Command::new("geckodriver")
            .spawn()
            .expect("Failed to start geckodriver");

        let desired_url = format!("https://debank.com/profile/{}", user_id);

        let c = ClientBuilder::native()
            .connect("http://localhost:4444")
            .await
            .expect("failed to connect to WebDriver");

        c.goto(desired_url.as_str()).await?;
        let url = c.current_url().await?;
        assert_eq!(url.as_ref(), desired_url.as_str());

        // Wait exactly 15 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

        let xpath = "//*[@id=\"root\"]/div[1]/div[2]/div[1]/div/div[2]/div/div[1]/div[2]/div[2]/div[1]/div[1]";
        let element = c.find(Locator::XPath(xpath)).await?;

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

        println!("Debank balance: {:?}", balance);
        let balance = balance
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!(format!("Failed to parse balance: {:?}", balance)))?;

        c.close().await?;
        driver_process.kill().expect("Failed to kill geckodriver");

        Ok(balance)
    }
}
