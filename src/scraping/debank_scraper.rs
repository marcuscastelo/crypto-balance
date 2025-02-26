use core::fmt;
use std::{collections::HashMap, time::Duration};

use error_stack::{bail, Context, IntoReport, IntoReportCompat, Result, ResultExt};
use fantoccini::{elements::Element, error::CmdError, Locator};
use reqwest::Url;

use super::{formatting::balance::format_balance, scraper_driver::ScraperDriver};

pub struct DebankBalanceScraper {
    driver: ScraperDriver,
}

#[derive(Debug)]
pub enum DebankScraperError {
    ElementNotFound,
    ElementTextNotFound,
    ElementHtmlNotFound,
    UnknownRowStructure,
    HeadersValuesLengthMismatch,
    UnknownHeader { header: String },
    UnknownTrackingType { tracking_type: String },
    UrlParseError,
    FailedToCreateDriver,
    FailedToNavigateToUrl,
    FailedToGetChainInfo,
    FailedToExploreTracking,
    FailedToExtractCellInfo,
    FailedToClickElement,
    GenericError,
}

impl fmt::Display for DebankScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Context for DebankScraperError {}

#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub name: String,
    pub wallet_info: Option<ChainWalletInfo>,
    pub project_info: Vec<ChainProjectInfo>,
}

#[derive(Debug, Clone)]
pub struct ChainWalletInfo {
    pub usd_value: String,
    pub tokens: Vec<SpotTokenInfo>,
}

#[derive(Debug, Clone)]
pub struct SpotTokenInfo {
    pub name: String,
    pub price: String,
    pub amount: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct ChainProjectInfo {
    pub name: String,
    pub trackings: Vec<ProjectTracking>,
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
    pub token_name: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct StakeTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LockedTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct RewardTokenInfo {
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct VestingTokenInfo {
    pub pool: String,
    pub balance: String,
    pub claimable_amount: Option<String>,
    pub end_time: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct YieldFarmTokenInfo {
    pub token_name: Option<String>,
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct DepositTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LiquidityPoolTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct FarmingTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

// Contains all fields from before, but optional
#[derive(Debug, Clone, Default)]
pub struct GenericTokenInfo {
    pub token_name: Option<String>,
    pub pool: Option<String>,
    pub balance: Option<String>,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub claimable_amount: Option<String>,
    pub end_time: Option<String>,
    pub usd_value: Option<String>,
    pub variant_header: Option<String>, // Supplied, Borrowed, Rewards, etc. (not to be confused with tracking title)
}

impl DebankBalanceScraper {
    pub async fn new() -> Result<Self, DebankScraperError> {
        let driver = ScraperDriver::new()
            .await
            .change_context(DebankScraperError::FailedToCreateDriver)?;
        Ok(Self { driver })
    }

    async fn open_debank_url(&self, user_id: &str) -> Result<(), DebankScraperError> {
        let url = Url::parse(format!("https://debank.com/profile/{}", user_id).as_str())
            .change_context(DebankScraperError::UrlParseError)?;
        self.driver
            .client
            .goto(url.as_str())
            .await
            .change_context(DebankScraperError::FailedToNavigateToUrl)
            .attach_printable("Failed to navigate to Debank URL")?;
        self.driver
            .client
            .wait()
            .for_url(url)
            .await
            .change_context(DebankScraperError::FailedToNavigateToUrl)
            .attach_printable("Timeout waiting for Debank URL")?;
        Ok(())
    }

    async fn wait_data_updated(&self) -> Result<(), DebankScraperError> {
        let update_xpath =
            "/html/body/div[1]/div[1]/div[1]/div/div/div/div[2]/div/div[2]/div[2]/span";

        let update_element = self
            .driver
            .client
            .wait()
            .for_element(Locator::XPath(update_xpath))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        let mut update_text = update_element
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        while !update_text.as_str().contains("Data updated") {
            tokio::time::sleep(Duration::from_secs(1)).await;
            log::trace!("Waiting for data to update...");
            update_text = update_element
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?;
        }
        Ok(())
    }

    async fn locate_chain_summary_elements(&self) -> Result<Vec<Element>, DebankScraperError> {
        let chains_selector = "div.AssetsOnChain_chainInfo__fKA2k";
        let chains = self
            .driver
            .client
            .find_all(Locator::Css(chains_selector))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        Ok(chains)
    }

    async fn get_chain_info(&self, chain: &Element) -> Result<ChainInfo, DebankScraperError> {
        let chain_name = chain
            .find(Locator::XPath("div[1]"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;
        log::info!("Getting chain balance for chain {}", chain_name);
        let chain_balance = chain
            .find(Locator::XPath("div[2]/span"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        chain
            .click()
            .await
            .change_context(DebankScraperError::FailedToClickElement)?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let token_wallet_selector = "div.TokenWallet_container__FUGTE";
        let project_selector = "div.Project_project__GCrhx";

        let wallet: Option<Element> = self
            .driver
            .client
            .find(Locator::Css(token_wallet_selector))
            .await
            .inspect_err(|e| log::error!("Failed to find wallet: {}", e))
            .ok();

        let projects: Vec<Element> = self
            .driver
            .client
            .find_all(Locator::Css(project_selector))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        let wallet_info = if let Some(wallet) = wallet.as_ref() {
            self.get_chain_wallet_info(wallet)
                .await
                .change_context(DebankScraperError::FailedToGetChainInfo)?
                .into()
        } else {
            None
        };

        let mut projects_info = Vec::new();
        for project in projects.iter() {
            let project_info = self
                .get_chain_project_info(project)
                .await
                .change_context(DebankScraperError::FailedToGetChainInfo)?;
            projects_info.push(project_info);
        }

        Ok(ChainInfo {
            name: chain_name,
            wallet_info,
            project_info: projects_info,
        })
    }

    async fn get_chain_wallet_info(
        &self,
        wallet: &Element,
    ) -> Result<ChainWalletInfo, DebankScraperError> {
        let usd_value = wallet
            .find(Locator::XPath("div[1]/div[2]"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        let token_table_body_xpath = "div[2]/div[1]/div[1]/div[2]";

        let table_body: Element = wallet
            .find(Locator::XPath(token_table_body_xpath))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        let token_rows = table_body
            .find_all(Locator::Css("div.db-table-wrappedRow"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        let mut tokens = Vec::new();

        let name_xpath = "div[1]/div[1]/div[1]/a";
        let price_xpath = "div[1]/div[2]";
        let amount_xpath = "div[1]/div[3]";
        let usd_value_xpath = "div[1]/div[4]";

        for row in token_rows {
            let name = row
                .find(Locator::XPath(name_xpath))
                .await
                .change_context(DebankScraperError::ElementNotFound)?
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?;
            tokens.push(SpotTokenInfo {
                name: row
                    .find(Locator::XPath(name_xpath))
                    .await
                    .change_context(DebankScraperError::ElementNotFound)?
                    .text()
                    .await
                    .change_context(DebankScraperError::ElementTextNotFound)?,
                price: row
                    .find(Locator::XPath(price_xpath))
                    .await
                    .change_context(DebankScraperError::ElementNotFound)?
                    .text()
                    .await
                    .change_context(DebankScraperError::ElementTextNotFound)?,
                amount: row
                    .find(Locator::XPath(amount_xpath))
                    .await
                    .change_context(DebankScraperError::ElementNotFound)?
                    .text()
                    .await
                    .change_context(DebankScraperError::ElementTextNotFound)?,
                usd_value: row
                    .find(Locator::XPath(usd_value_xpath))
                    .await
                    .change_context(DebankScraperError::ElementHtmlNotFound)?
                    .text()
                    .await
                    .change_context(DebankScraperError::ElementTextNotFound)?,
            });
        }

        Ok(ChainWalletInfo { usd_value, tokens })
    }

    async fn get_chain_project_info(
        &self,
        project: &Element,
    ) -> Result<ChainProjectInfo, DebankScraperError> {
        let name = project
            .find(Locator::XPath("div[1]/div[1]/div[2]/span"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        let tracking_elements = project
            .find_all(Locator::Css("div.Panel_container__Vltd1"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?;

        let mut trackings = Vec::new();

        for tracking_element in tracking_elements.iter() {
            let tracking = self
                .explore_tracking(tracking_element)
                .await
                .change_context(DebankScraperError::FailedToExploreTracking)?;
            trackings.push(tracking);
        }

        Ok(ChainProjectInfo { name, trackings })
    }

    async fn explore_tracking(
        &self,
        tracking: &Element,
    ) -> Result<ProjectTracking, DebankScraperError> {
        let tracking_type = tracking
            .find(Locator::XPath("div[1]/div[1]/div[1]"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        let tables = tracking
            .find_all(Locator::XPath("div[2]/div"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?;

        let mut generic_infos: Vec<Vec<(String, String)>> = Vec::new();

        for table in tables {
            let tracking_headers = table
                .find_all(Locator::XPath("div[1]//span"))
                .await
                .change_context(DebankScraperError::ElementHtmlNotFound)?;

            let headers = futures::future::join_all(
                tracking_headers
                    .iter()
                    .map(|header| async {
                        header
                            .text()
                            .await
                            .change_context(DebankScraperError::ElementTextNotFound)
                    })
                    .collect::<Vec<_>>(),
            )
            .await;

            // Convert all results to strings returning an error if any of them fails
            let headers = headers
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .change_context(DebankScraperError::GenericError)?;

            let tracking_body = table
                .find(Locator::XPath("div[2]"))
                .await
                .change_context(DebankScraperError::ElementNotFound)?;

            let row_selector = "div.table_contentRow__Mi3k5.flex_flexRow__y0UR2";
            let rows = tracking_body
                .find_all(Locator::Css(row_selector))
                .await
                .change_context(DebankScraperError::ElementNotFound)?;

            for row in rows.as_slice() {
                let cells = row
                    .find_all(Locator::XPath("div"))
                    .await
                    .change_context(DebankScraperError::ElementNotFound)?;
                let mut values = Vec::new();
                for cell in cells.as_slice() {
                    let cell_info = self.extract_cell_info(cell).await?;
                    values.push(cell_info);
                }

                if headers.len() != values.len() {
                    bail!(DebankScraperError::HeadersValuesLengthMismatch);
                }

                let zipped = headers
                    .clone()
                    .into_iter()
                    .zip(values.into_iter())
                    .collect::<Vec<_>>();

                generic_infos.push(zipped);
            }
        }

        let generic_infos = self.parse_generic_info(generic_infos)?;

        let specialized = self.specialize_generic_info(&tracking_type, generic_infos)?;

        Ok(specialized)
    }

    fn specialize_generic_info(
        &self,
        tracking_type: &str,
        generic_infos: Vec<GenericTokenInfo>,
    ) -> Result<ProjectTracking, DebankScraperError> {
        match tracking_type {
            "Yield" => Ok(ProjectTracking::YieldFarm {
                yield_farm: generic_infos
                    .into_iter()
                    .map(|generic| YieldFarmTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                    })
                    .collect(),
            }),
            "Staked" => Ok(ProjectTracking::Staked {
                staked: generic_infos
                    .into_iter()
                    .map(|generic| StakeTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        rewards: generic.rewards,
                        usd_value: generic.usd_value.expect("USD value not found"),
                    })
                    .collect(),
            }),
            "Deposit" => Ok(ProjectTracking::Deposit {
                deposit: generic_infos
                    .into_iter()
                    .map(|generic| DepositTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                    })
                    .collect(),
            }),
            "Locked" => Ok(ProjectTracking::Locked {
                locked: generic_infos
                    .into_iter()
                    .map(|generic| LockedTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        unlock_time: generic.unlock_time,
                        rewards: generic.rewards,
                        usd_value: generic.usd_value.expect("USD value not found"),
                    })
                    .collect(),
            }),
            "Vesting" => Ok(ProjectTracking::Vesting {
                vesting: generic_infos
                    .into_iter()
                    .map(|generic| VestingTokenInfo {
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                        end_time: generic.end_time.expect("End time not found"),
                        claimable_amount: generic.claimable_amount,
                    })
                    .collect(),
            }),
            "Rewards" => Ok(ProjectTracking::Rewards {
                rewards: generic_infos
                    .into_iter()
                    .map(|generic| RewardTokenInfo {
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                    })
                    .collect(),
            }),
            "Liquidity Pool" => Ok(ProjectTracking::LiquidityPool {
                liquidity_pool: generic_infos
                    .into_iter()
                    .map(|generic| LiquidityPoolTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                        rewards: generic.rewards,
                    })
                    .collect(),
            }),
            "Farming" => Ok(ProjectTracking::Farming {
                farming: generic_infos
                    .into_iter()
                    .map(|generic| FarmingTokenInfo {
                        token_name: generic.token_name,
                        pool: generic.pool.expect("Pool not found"),
                        balance: generic.balance.expect("Balance not found"),
                        usd_value: generic.usd_value.expect("USD value not found"),
                        rewards: generic.rewards,
                    })
                    .collect(),
            }),
            "Lending" => Ok(ProjectTracking::Lending {
                supplied: generic_infos
                    .iter()
                    .filter(|generic| generic.variant_header == Some("Supplied".into()))
                    .map(|generic| LendingTokenInfo {
                        token_name: generic
                            .token_name
                            .as_ref()
                            .expect("Token name not found")
                            .clone(),
                        balance: generic.balance.as_ref().expect("Balance not found").clone(),
                        usd_value: generic
                            .usd_value
                            .as_ref()
                            .expect("USD value not found")
                            .clone(),
                    })
                    .collect(),
                borrowed: generic_infos
                    .iter()
                    .filter(|generic| generic.variant_header == Some("Borrowed".into()))
                    .map(|generic| LendingTokenInfo {
                        token_name: generic
                            .token_name
                            .as_ref()
                            .expect("Token name not found")
                            .clone(),
                        balance: generic.balance.as_ref().expect("Balance not found").clone(),
                        usd_value: generic
                            .usd_value
                            .as_ref()
                            .expect("USD value not found")
                            .clone(),
                    })
                    .collect::<Vec<_>>()
                    .into(),
                rewards: generic_infos
                    .iter()
                    .filter(|generic| generic.variant_header == Some("Rewards".into()))
                    .map(|generic| LendingTokenInfo {
                        token_name: generic
                            .token_name
                            .as_ref()
                            .expect("Token name not found")
                            .clone(),
                        balance: generic.balance.as_ref().expect("Balance not found").clone(),
                        usd_value: generic
                            .usd_value
                            .as_ref()
                            .expect("USD value not found")
                            .clone(),
                    })
                    .collect::<Vec<_>>()
                    .into(),
            }),
            _ => Err(DebankScraperError::UnknownTrackingType {
                tracking_type: tracking_type.into(),
            })
            .change_context(DebankScraperError::GenericError)?,
        }
    }

    fn parse_generic_info(
        &self,
        generic_info: Vec<Vec<(String, String)>>,
    ) -> Result<Vec<GenericTokenInfo>, DebankScraperError> {
        let mut infos = Vec::new();
        for row_values in generic_info.as_slice() {
            let mut info = GenericTokenInfo::default();
            for (header, value) in row_values.as_slice() {
                match header.as_str() {
                    " " => info.token_name = value.clone().into(),
                    "Pool" => info.pool = value.clone().into(),
                    "Balance" => info.balance = value.clone().into(),
                    "Unlock time" => info.unlock_time = value.clone().into(),
                    "Claimable Amount" => info.claimable_amount = value.clone().into(),
                    "End Time" => info.end_time = value.clone().into(),
                    "USD Value" => info.usd_value = value.clone().into(),
                    "Supplied" | "Borrowed" | "Rewards" => {
                        // Variant header
                        info.variant_header = header.clone().into();
                        info.token_name = value.clone().into();
                        if header == "Rewards" {
                            info.rewards = value.clone().into();
                        }
                    }
                    _ => {
                        log::error!("Unknown header: {}", header);
                        return Err(DebankScraperError::UnknownHeader {
                            header: header.clone(),
                        }
                        .into());
                    }
                }
            }
            infos.push(info);
        }

        Ok(infos)
    }

    async fn extract_cell_info(&self, cell: &Element) -> Result<String, DebankScraperError> {
        let simple_span = cell.find(Locator::XPath("span")).await;

        if let Ok(span) = simple_span {
            return Ok(span
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?);
        }

        let span_div_and_a = cell.find(Locator::XPath("span/div")).await;

        if let Ok(div_span_div_and_a) = span_div_and_a {
            let div_span_div_and_a_text = div_span_div_and_a
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?;
            let div_span_div_and_a_a = div_span_div_and_a
                .find(Locator::XPath("a"))
                .await
                .change_context(DebankScraperError::ElementNotFound)?
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?;

            return Ok(format!(
                "{} {}", // 0.001 ETH
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
                .await
                .change_context(DebankScraperError::ElementNotFound)?
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?);
        }

        bail!(DebankScraperError::FailedToExtractCellInfo)
    }
}

impl DebankBalanceScraper {
    pub async fn get_total_balance(&self, user_id: &str) -> Result<f64, DebankScraperError> {
        self.open_debank_url(user_id).await?;
        self.wait_data_updated().await?;

        let xpath = "//*[@id=\"root\"]/div[1]/div[1]/div/div/div/div[2]/div/div[1]/div[2]/div[2]/div[1]/div[1]";
        let balance_text = self
            .driver
            .client
            .find(Locator::XPath(xpath))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        format_balance(&balance_text)
            .into_report()
            .map_err(|e| {
                log::error!("Failed to parse balance: {}", e);
                DebankScraperError::GenericError
            })
            .into_report()
    }

    pub async fn get_chain_infos(
        &self,
        user_id: &str,
    ) -> Result<HashMap<String, ChainInfo>, DebankScraperError> {
        self.open_debank_url(user_id).await?;
        self.wait_data_updated().await?;
        let chain_summaries: Vec<Element> = self.locate_chain_summary_elements().await?;

        let mut chain_infos = HashMap::new();

        for chain in chain_summaries.as_slice() {
            let chain_info = self.get_chain_info(chain).await?;
            chain_infos.insert(chain_info.name.clone(), chain_info);
        }

        Ok(chain_infos)
    }
}
