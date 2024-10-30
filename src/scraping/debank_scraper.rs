use super::{formatting::balance::format_balance, simple_xpath_scraper::SimpleXPathScraper};

pub struct DebankBalanceScraper;
impl DebankBalanceScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        let balance_text = SimpleXPathScraper {
            url: format!("https://debank.com/profile/{}", user_id),
            xpath: "//*[@id=\"root\"]/div[1]/div[1]/div/div/div/div[2]/div/div[1]/div[2]/div[2]/div[1]/div[1]".to_owned(),
            wait_time_secs: 15,
        }.scrape().await?;

        format_balance(&balance_text)
    }
}
