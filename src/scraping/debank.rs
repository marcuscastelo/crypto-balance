use crate::simple_balance_scrapper::SimpleBalanceScrapper;

pub struct DebankScraper;
//*[@id="root"]/section/section/main/div/div[1]/div[2]/div/div/div[1]/div/div/div[1]/span[1]
impl DebankScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        SimpleBalanceScrapper {
            url: format!("https://debank.com/profile/{}", user_id),
            xpath: "//*[@id=\"root\"]/div[1]/div[1]/div/div/div/div[2]/div/div[1]/div[2]/div[2]/div[1]/div[1]".to_owned(),
            wait_time: 15,
        }.scrape().await
    }
}
