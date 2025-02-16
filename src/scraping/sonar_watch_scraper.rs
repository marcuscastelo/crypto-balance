use std::time::Duration;

use reqwest::Url;

use super::scraper_driver::ScraperDriver;

pub struct SonarWatchScraper {
    driver: ScraperDriver,
}

impl SonarWatchScraper {
    pub async fn new() -> anyhow::Result<Self> {
        let driver = ScraperDriver::new().await?;
        Ok(Self { driver })
    }
}

impl SonarWatchScraper {
    pub async fn open_sonar_watch_url(&self, user_id: &str) -> anyhow::Result<()> {
        // https://sonar.watch/portfolio/3yy1dGAXHDRqGRwaVP3GfEpvSqDs251EiKN79NquewaR
        let url = Url::parse(format!("https://sonar.watch/ddddd/{}", user_id).as_str())?;
        self.driver.client.goto(url.as_str()).await?;
        self.driver
            .client
            .wait()
            .at_most(Duration::from_secs(100000))
            .for_url(url)
            .await?;
        Ok(())
    }
}

impl SonarWatchScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        self.open_sonar_watch_url(user_id).await?;

        Ok(0f64)
    }
}
