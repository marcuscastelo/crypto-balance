use crate::simple_xpath_scraper::SimpleXPathScraper;

use super::{backend_scraper::scrape, formatting::balance::format_balance};

pub struct SonarWatchScraper;

impl SonarWatchScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        // let balance_text = SimpleXPathScraper {
        //     url: format!("https://sonar.watch/portfolio/{}", user_id),
        //     xpath: "//*[@id=\"root\"]/section/section/main/div/div[1]/div[2]/div/div/div[1]/div/div/div[1]/span[1]".to_owned(),
        //     wait_time_secs: 60,
        // }.scrape().await?;

        scrape().await
    }
}
