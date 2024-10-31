use std::time::Duration;

use fantoccini::{elements::Element, Locator};
use reqwest::Url;

use super::scraper_driver::ScraperDriver;

pub struct DebankBalanceScraper {
    driver: ScraperDriver,
}

#[derive(Debug, Clone)]
pub struct ChainWalletInfo {
    usd_value: String,
    tokens: Vec<SpotTokenInfo>,
}

#[derive(Debug, Clone)]
pub struct SpotTokenInfo {
    token_name: String,
    price: String,
    amount: String,
    usd_value: String,
}

impl DebankBalanceScraper {
    pub async fn new() -> anyhow::Result<Self> {
        let driver = ScraperDriver::new().await?;
        Ok(Self { driver })
    }

    async fn open_debank_url(&self, user_id: &str) -> anyhow::Result<()> {
        let url = Url::parse(format!("https://debank.com/profile/{}", user_id).as_str())?;
        self.driver.client.goto(url.as_str()).await?;
        self.driver.client.wait().for_url(url).await?;
        Ok(())
    }

    async fn wait_data_updated(&self) -> anyhow::Result<()> {
        let update_xpath =
            "/html/body/div[1]/div[1]/div[1]/div/div/div/div[2]/div/div[2]/div[2]/span";

        let update_element = self
            .driver
            .client
            .wait()
            .for_element(Locator::XPath(update_xpath))
            .await?;

        let mut update_text = update_element.text().await?;

        while !update_text.as_str().contains("Data updated") {
            tokio::time::sleep(Duration::from_secs(1)).await;
            log::trace!("Waiting for data to update...");
            update_text = update_element.text().await?;
        }
        Ok(())
    }

    async fn locate_chain_summary_elements(&self) -> anyhow::Result<Vec<Element>> {
        let chains_selector = "div.AssetsOnChain_chainInfo__fKA2k";
        let chains = self
            .driver
            .client
            .find_all(Locator::Css(chains_selector))
            .await?;

        Ok(chains)
    }

    async fn get_chain_balances(&self, chain: &Element) -> anyhow::Result<()> {
        let chain_name = chain.find(Locator::XPath("div[1]")).await?.text().await?;
        log::info!("Getting chain balance for chain {}", chain_name);
        let chain_balance = chain
            .find(Locator::XPath("div[2]/span"))
            .await?
            .text()
            .await?;
        log::info!("{}: Chain balance: {}", chain_name, chain_balance);

        chain.click().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let token_wallet_selector = "div.TokenWallet_container__FUGTE";
        let project_selector = "div.Project_project__GCrhx";

        let wallet: Option<Element> = self
            .driver
            .client
            .wait()
            .for_element(Locator::Css(token_wallet_selector))
            .await
            .ok(); // TODO: instead of timeout, detect when wallet has 0 balance and skip waiting for element sinc

        let projects: Vec<Element> = self
            .driver
            .client
            .find_all(Locator::Css(project_selector))
            .await?;

        if let Some(wallet) = wallet.as_ref() {
            let wallet_info = self.get_chain_wallet_info(wallet).await?;
            log::info!("{}: Wallet info: {:#?}", chain_name, wallet_info);
        }

        let mut project_names = Vec::new();
        for project in projects.iter() {
            let project_name = project
                .find(Locator::XPath("div[1]/div[1]/div[2]/span"))
                .await?
                .text()
                .await?;
            project_names.push(project_name);
        }

        log::info!("{}: Projects: {:?}", chain_name, project_names);

        Ok(())
    }

    async fn get_chain_wallet_info(&self, wallet: &Element) -> anyhow::Result<ChainWalletInfo> {
        let usd_value = wallet
            .find(Locator::XPath("div[1]/div[2]"))
            .await?
            .text()
            .await?;

        let token_table_body_xpath = "div[2]/div[1]/div[1]/div[2]";

        log::trace!("Locating token table body");
        let table_body: Element = wallet.find(Locator::XPath(token_table_body_xpath)).await?;

        let token_rows = table_body
            .find_all(Locator::Css("div.db-table-wrappedRow"))
            .await?;

        let mut tokens = Vec::new();

        let name_xpath = "div[1]/div[1]/div[1]/a";
        let price_xpath = "div[1]/div[2]";
        let amount_xpath = "div[1]/div[3]";
        let usd_value_xpath = "div[1]/div[4]";

        for row in token_rows {
            let name = row.find(Locator::XPath(name_xpath)).await?.text().await?;
            log::trace!("Found token on wallet: {}", name);
            tokens.push(SpotTokenInfo {
                token_name: row.find(Locator::XPath(name_xpath)).await?.text().await?,
                price: row.find(Locator::XPath(price_xpath)).await?.text().await?,
                amount: row.find(Locator::XPath(amount_xpath)).await?.text().await?,
                usd_value: row
                    .find(Locator::XPath(usd_value_xpath))
                    .await?
                    .text()
                    .await?,
            });
            log::trace!("Token: {:#?}", tokens.last().unwrap());
        }

        Ok(ChainWalletInfo { usd_value, tokens })
    }
}

impl DebankBalanceScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> anyhow::Result<f64> {
        self.open_debank_url(user_id).await?;
        self.wait_data_updated().await?;
        let chain_summaries: Vec<Element> = self.locate_chain_summary_elements().await?;
        for chain in chain_summaries.as_slice() {
            self.get_chain_balances(chain).await?;
        }

        // let xpath = "//*[@id=\"root\"]/div[1]/div[1]/div/div/div/div[2]/div/div[1]/div[2]/div[2]/div[1]/div[1]";
        for _ in 0..150 {
            // progress.inc(1);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // format_balance(&balance_text)
        Ok(0f64)
    }
}
