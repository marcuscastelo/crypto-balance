use crate::simple_balance_scrapper::SimpleBalanceScrapper;

pub struct StepSVMScraper;

impl StepSVMScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        SimpleBalanceScrapper {
            url: format!("https://app.step.finance/en/dashboard?watching={}", user_id),
            xpath: "//*[@id=\"root\"]/section/section/main/div/div[1]/div[2]/div/div/div[1]/div/div/div[1]/span[1]".to_owned(),
            wait_time: 25,
        }.scrape().await
    }
}
