use fantoccini::Locator;
use indicatif::ProgressBar;
use undetected_chromedriver::chrome;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    scraper_driver::ScraperDriver,
};

pub struct SimpleXPathScraper {
    pub url: String,
    pub xpath: String,
    pub wait_time_secs: u64,
}

impl SimpleXPathScraper {
    pub async fn scrape(&self) -> anyhow::Result<String> {
        let scraper = ScraperDriver::new().await?;

        let client = &scraper.client;
        // let client = chrome().await.expect("Failed to start chrome");

        log::trace!("Navigating to {}", self.url);
        client.goto(self.url.as_str()).await?;
        let url = client.current_url().await?;

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
        let element = client.find(Locator::XPath(self.xpath.as_str())).await?;

        log::trace!("Getting text from element");
        let text = element.text().await?;

        log::trace!("Returning balance: {}", text);
        Ok(text)
    }
}
