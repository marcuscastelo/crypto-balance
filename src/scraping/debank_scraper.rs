use std::{str::FromStr, time::Duration};

use fantoccini::{elements::Element, error::CmdError, Locator};
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
    name: String,
    price: String,
    amount: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct ChainProjectInfo {
    name: String,
    trackings: Vec<ProjectTracking>,
}

#[derive(Debug, Clone)]
pub enum ProjectTracking {
    Lending {
        supplied: Vec<LendingTokenInfo>,
        borrowed: Option<Vec<LendingTokenInfo>>,
        rewards: Option<Vec<LendingTokenInfo>>,
    },
    Staked {
        staked: Vec<StakeTokenInfo>,
    },
    Locked {
        locked: Vec<LockedTokenInfo>,
    },
    Rewards {
        rewards: Vec<RewardTokenInfo>,
    },
    Vesting {
        vesting: Vec<VestingTokenInfo>,
    },
    YieldFarm {
        yield_farm: Vec<YieldFarmTokenInfo>,
    },
    Deposit {
        deposit: Vec<DepositTokenInfo>,
    },
    LiquidityPool {
        liquidity_pool: Vec<LiquidityPoolTokenInfo>,
    },
    Farming {
        farming: Vec<FarmingTokenInfo>,
    },
    Generic {
        info: Vec<GenericTokenInfo>,
    },
}

#[derive(Debug, Clone)]
pub struct LendingTokenInfo {
    token_name: String,
    balance: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct StakeTokenInfo {
    token_name: Option<String>, // When token_name is not available, pool name is used
    pool: String,
    balance: String,
    rewards: Option<String>,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LockedTokenInfo {
    token_name: Option<String>, // When token_name is not available, pool name is used
    pool: String,
    balance: String,
    rewards: Option<String>,
    unlock_time: Option<String>,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct RewardTokenInfo {
    pool: String,
    balance: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct VestingTokenInfo {
    pool: String,
    balance: String,
    claimable_amount: Option<String>,
    end_time: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct YieldFarmTokenInfo {
    token_name: Option<String>,
    pool: String,
    balance: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct DepositTokenInfo {
    token_name: Option<String>, // When token_name is not available, pool name is used
    pool: String,
    balance: String,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LiquidityPoolTokenInfo {
    token_name: Option<String>, // When token_name is not available, pool name is used
    pool: String,
    balance: String,
    rewards: Option<String>,
    usd_value: String,
}

#[derive(Debug, Clone)]
pub struct FarmingTokenInfo {
    token_name: Option<String>, // When token_name is not available, pool name is used
    pool: String,
    balance: String,
    rewards: Option<String>,
    usd_value: String,
}

// Contains all fields from before, but optional
#[derive(Debug, Clone)]
pub struct GenericTokenInfo {
    token_name: Option<String>,
    pool: Option<String>,
    balance: Option<String>,
    rewards: Option<String>,
    unlock_time: Option<String>,
    claimable_amount: Option<String>,
    end_time: Option<String>,
    usd_value: Option<String>,
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
            .find(Locator::Css(token_wallet_selector))
            .await
            .ok();

        let projects: Vec<Element> = self
            .driver
            .client
            .find_all(Locator::Css(project_selector))
            .await?;

        if let Some(wallet) = wallet.as_ref() {
            let wallet_info = self.get_chain_wallet_info(wallet).await?;
            log::info!("{}: Wallet info: {:#?}", chain_name, wallet_info);
        } else {
            log::info!("{}: No wallet found", chain_name);
        }

        let mut projects_info = Vec::new();
        for project in projects.iter() {
            let project_info = self.get_chain_project_info(project).await?;
            projects_info.push(project_info);
        }

        log::info!("{}: Projects: {:#?}", chain_name, projects_info);

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
                name: row.find(Locator::XPath(name_xpath)).await?.text().await?,
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

    async fn get_chain_project_info(&self, project: &Element) -> anyhow::Result<ChainProjectInfo> {
        let name = project
            .find(Locator::XPath("div[1]/div[1]/div[2]/span"))
            .await?
            .text()
            .await?;

        log::trace!("Project name: {}", name);

        let tracking_elements = project
            .find_all(Locator::Css("div.Panel_container__Vltd1"))
            .await?;

        let mut trackings = Vec::new();

        for tracking_element in tracking_elements.iter() {
            let tracking = self.get_project_tracking(tracking_element).await?;
            trackings.push(tracking);
        }

        Ok(ChainProjectInfo { name, trackings })
    }

    async fn get_project_tracking(&self, tracking: &Element) -> anyhow::Result<ProjectTracking> {
        let tracking_type = tracking
            .find(Locator::XPath("div[1]/div[1]/div[1]"))
            .await?
            .text()
            .await?;

        log::trace!("Tracking type: {}", tracking_type);

        let tracking_header = tracking
            .find(Locator::XPath("div[2]/div[1]/div[1]"))
            .await?;

        let tracking_headers = tracking_header.find_all(Locator::XPath("div/span")).await?;

        let headers = futures::future::join_all(
            tracking_headers
                .iter()
                .map(|header| header.text())
                .collect::<Vec<_>>(),
        )
        .await;

        // Convert all results to strings returning an error if any of them fails
        let headers = headers.into_iter().collect::<Result<Vec<_>, _>>()?;

        for header in headers.iter() {
            log::trace!("Header: {:?}", header);
        }

        let tracking_body = tracking
            .find(Locator::XPath("div[2]/div[1]/div[2]"))
            .await?;

        let row_selector = "div.table_contentRow__Mi3k5.flex_flexRow__y0UR2";
        let rows = tracking_body.find_all(Locator::Css(row_selector)).await?;

        let mut generic_infos: Vec<(String, String)> = Vec::new();

        for row in rows.as_slice() {
            let cells = row.find_all(Locator::XPath("div")).await?;
            let mut values = Vec::new();
            for cell in cells.as_slice() {
                log::trace!("Cell: {:?}", cell.html(true).await?);
                let cell_info = self.extract_cell_info(cell).await?;
                log::trace!("Cell info: {}", cell_info);
                values.push(cell_info);
            }

            if headers.len() != values.len() {
                return Err(anyhow::anyhow!("Headers and values length mismatch"));
            }
            generic_infos.append(
                headers
                    .clone()
                    .into_iter()
                    .zip(values.into_iter())
                    .collect::<Vec<_>>()
                    .as_mut(),
            );
        }

        for info in generic_infos.iter() {
            log::trace!("Info: {:?}", info);
        }

        Ok(ProjectTracking::Deposit {
            deposit: Vec::new(),
        })
    }

    async fn extract_cell_info(&self, cell: &Element) -> anyhow::Result<String> {
        let simple_span = cell.find(Locator::XPath("span")).await;

        if let Ok(span) = simple_span {
            return Ok(span.text().await?);
        }

        let span_div_and_a = cell.find(Locator::XPath("span/div")).await;

        if let Ok(div_span_div_and_a) = span_div_and_a {
            let div_span_div_and_a_text = div_span_div_and_a.text().await?;
            let div_span_div_and_a_a = div_span_div_and_a
                .find(Locator::XPath("a"))
                .await?
                .text()
                .await?;

            return Ok(format!(
                "{};;{}", // 0.001;;ETH
                div_span_div_and_a_text, div_span_div_and_a_a
            ));
        }

        let label_with_icon = cell
            .find(Locator::Css(
                "div.Flex_flex__KFQty Flex_flexRow__jNYOK.LabelWithIcon_container__-yKOy",
            ))
            .await;

        if let Ok(label_with_icon) = label_with_icon {
            return Ok(label_with_icon
                .find(Locator::XPath("div[2]/a"))
                .await?
                .text()
                .await?);
        }

        Err(anyhow::anyhow!(
            "Unknown row structure: maybe debank changed the layout!"
        ))
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
