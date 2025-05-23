use core::fmt;
use std::{collections::HashMap, fmt::Debug, time::Duration};

use error_stack::{bail, Context, Result, ResultExt};
use fantoccini::{elements::Element, Locator};
use futures::join;
use reqwest::Url;
use tracing::{event, info, instrument, Instrument, Level};

use super::{formatting::balance::format_balance, scraper_driver::ScraperDriver};

pub struct DebankBalanceScraper {
    driver: ScraperDriver,
}

impl Debug for DebankBalanceScraper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebankBalanceScraper {{}}")
    }
}

#[derive(Debug)]
pub enum DebankScraperError {
    ElementNotFound,
    ElementTextNotFound,
    ElementHtmlNotFound,
    HeadersValuesLengthMismatch,
    UnknownHeader,
    UnknownTrackingType,
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
#[allow(dead_code)]
pub struct ChainInfo {
    pub name: String,
    pub wallet_info: Option<ChainWalletInfo>,
    pub project_info: Vec<ChainProjectInfo>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChainWalletInfo {
    pub usd_value: String,
    pub tokens: Vec<SpotTokenInfo>,
}

impl fmt::Display for ChainWalletInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens = self
            .tokens
            .iter()
            .map(|x| format!("{} {}: {}", x.amount, x.name, x.usd_value))
            .reduce(|a, b| format!("{}; {}", a, b))
            .unwrap_or("<no tokens>".to_string());

        write!(f, "Wallet: {} USD\nTokens: {}", self.usd_value, tokens)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpotTokenInfo {
    pub name: String,
    pub price: String,
    pub amount: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChainProjectInfo {
    pub name: String,
    pub trackings: Vec<ProjectTracking>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct LendingTokenInfo {
    pub token_name: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StakeTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LockedTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RewardTokenInfo {
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VestingTokenInfo {
    pub pool: String,
    pub balance: String,
    pub claimable_amount: Option<String>,
    pub end_time: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct YieldFarmTokenInfo {
    pub token_name: Option<String>,
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DepositTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LiquidityPoolTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    #[instrument]
    pub async fn new() -> Result<Self, DebankScraperError> {
        let driver = ScraperDriver::new()
            .await
            .change_context(DebankScraperError::FailedToCreateDriver)?;
        Ok(Self { driver })
    }

    #[instrument]
    async fn open_debank_url(&self, user_id: &str) -> Result<(), DebankScraperError> {
        let url = Url::parse(format!("https://debank.com/profile/{}", user_id).as_str())
            .change_context(DebankScraperError::UrlParseError)?;

        event!(Level::DEBUG, url = %url, "Opening Debank URL");
        self.driver
            .client
            .goto(url.as_str())
            .await
            .change_context(DebankScraperError::FailedToNavigateToUrl)
            .attach_printable("Failed to navigate to Debank URL")?;

        event!(Level::DEBUG, "Waiting for Debank URL to load");
        self.driver
            .client
            .wait()
            .for_url(url)
            .await
            .change_context(DebankScraperError::FailedToNavigateToUrl)
            .attach_printable("Timeout waiting for Debank URL")?;

        event!(Level::DEBUG, "Debank URL loaded");
        Ok(())
    }

    #[instrument]
    async fn wait_data_updated(&self) -> Result<(), DebankScraperError> {
        let update_xpath =
            "/html/body/div[1]/div[1]/div[1]/div/div/div/div[2]/div/div[2]/div[2]/span";

        tracing::trace!("Locating update element...");
        let update_element = self
            .driver
            .client
            .wait()
            .for_element(Locator::XPath(update_xpath))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        tracing::trace!("Fetching update text...");
        let mut update_text = update_element
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        while !update_text.as_str().contains("Data updated") {
            tracing::trace!("Waiting for data to update...");
            tokio::time::sleep(Duration::from_millis(100)).await;
            update_text = update_element
                .text()
                .await
                .change_context(DebankScraperError::ElementTextNotFound)?;
            tracing::trace!("Update text: {}", update_text);
        }
        tracing::trace!("Data updated");
        Ok(())
    }

    #[instrument]
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

    #[instrument]
    async fn get_chain_info(&self, chain: &Element) -> Result<ChainInfo, DebankScraperError> {
        let chain_name = chain
            .find(Locator::XPath("div[1]"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;
        info!(chain_name = %chain_name, "Obtained chain name");

        chain
            .click()
            .await
            .change_context(DebankScraperError::FailedToClickElement)?;

        let token_wallet_selector = "div.TokenWallet_container__FUGTE";
        let project_selector = "div.Project_project__GCrhx";

        let (wallet_result, projects_result) = join!(
            self.driver.client.find(Locator::Css(token_wallet_selector)),
            self.driver.client.find_all(Locator::Css(project_selector))
        );

        let wallet: Option<Element> = wallet_result
            .inspect_err(|e| tracing::error!("Failed to find wallet: {}", e))
            .ok();

        let projects: Vec<Element> =
            projects_result.change_context(DebankScraperError::ElementNotFound)?;

        return self
            .get_chain_info_parallel(chain_name, wallet.as_ref(), projects)
            .await;
    }

    async fn get_chain_info_parallel(
        &self,
        chain_name: String,
        wallet: Option<&Element>,
        projects: Vec<Element>,
    ) -> Result<ChainInfo, DebankScraperError> {
        let wallet_info = if let Some(wallet) = wallet.as_ref() {
            self.get_chain_wallet_info(wallet)
                .await
                .change_context(DebankScraperError::FailedToGetChainInfo)?
                .into()
        } else {
            None
        };

        let projects_info = futures::future::join_all(
            projects
                .iter()
                .map(|project| self.get_chain_project_info(project)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .change_context(DebankScraperError::FailedToGetChainInfo)?;

        Ok(ChainInfo {
            name: chain_name,
            wallet_info,
            project_info: projects_info,
        })
    }

    #[instrument]
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

        let tokens = futures::future::join_all(
            token_rows
                .iter()
                .map(|row| self.get_chain_wallet_row_info(row)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

        Ok(ChainWalletInfo { usd_value, tokens })
    }

    async fn get_chain_wallet_row_info(
        &self,
        row: &Element,
    ) -> Result<SpotTokenInfo, DebankScraperError> {
        const NAME_XPATH: &'static str = "div[1]/div[1]/div[1]/a";
        const PRICE_XPATH: &'static str = "div[1]/div[2]";
        const AMOUNT_XPATH: &'static str = "div[1]/div[3]";
        const USD_VALUE_XPATH: &'static str = "div[1]/div[4]";

        let (name, price, amount, usd_value) = join!(
            self.get_chain_wallet_row_field(row, NAME_XPATH),
            self.get_chain_wallet_row_field(row, PRICE_XPATH),
            self.get_chain_wallet_row_field(row, AMOUNT_XPATH),
            self.get_chain_wallet_row_field(row, USD_VALUE_XPATH)
        );

        Ok(SpotTokenInfo {
            name: name?,
            price: price?,
            amount: amount?,
            usd_value: usd_value?,
        })
    }
    async fn get_chain_wallet_row_field(
        &self,
        row: &Element,
        xpath: &str,
    ) -> Result<String, DebankScraperError> {
        let field = row
            .find(Locator::XPath(xpath))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;
        Ok(field)
    }

    #[instrument]
    async fn get_chain_project_info(
        &self,
        project: &Element,
    ) -> Result<ChainProjectInfo, DebankScraperError> {
        let name = {
            // TODO: Move this retry logic to a common function if it really solves the problem (under testing for now)
            let mut name = None;
            let mut retries = 3;
            while retries > 0 {
                retries -= 1;
                let element = project
                    .find(Locator::XPath("div[1]/div[1]/div[2]/span"))
                    .await
                    .change_context(DebankScraperError::ElementHtmlNotFound);

                let new_name = match element {
                    Ok(element) => element
                        .text()
                        .await
                        .change_context(DebankScraperError::ElementTextNotFound),
                    Err(e) => Err(e),
                };

                match new_name {
                    Ok(new_name) => {
                        name = Some(new_name);
                        break;
                    }
                    Err(report) => {
                        let span = tracing::span!(Level::DEBUG, "waiting_retry_name", retries);
                        let _enter = span.enter();
                        tracing::error!("Failed to get project name: {:?}", report);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
            }

            name.ok_or(DebankScraperError::ElementTextNotFound)?
        };

        let tracking_elements = project
            .find_all(Locator::Css("div.Panel_container__Vltd1"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?;

        let tracking_futures = tracking_elements
            .iter()
            .map(|tracking_element| self.explore_tracking(tracking_element));

        let trackings = futures::future::join_all(tracking_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .change_context(DebankScraperError::FailedToExploreTracking)?;

        Ok(ChainProjectInfo { name, trackings })
    }

    #[instrument]
    async fn explore_tracking(
        &self,
        tracking: &Element,
    ) -> Result<ProjectTracking, DebankScraperError> {
        tracing::trace!("Fetching tracking type...");
        let tracking_type = tracking
            .find(Locator::XPath("div[1]/div[1]/div[1]"))
            .instrument(tracing::span!(Level::DEBUG, "tracking_type"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        tracing::trace!("Tracking type: {}", tracking_type);

        tracing::trace!("Locating tracking tables...");
        let tables = tracking
            .find_all(Locator::XPath("div[2]/div"))
            .instrument(tracing::span!(Level::DEBUG, "tracking_tables"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?;

        tracing::trace!("Found {} tables", tables.len());

        let generic_infos = self.extract_generic_infos_from_tables(tables).await?;

        let specialized = self.specialize_generic_info(&tracking_type, generic_infos)?;

        Ok(specialized)
    }

    #[instrument(skip(self, tables), fields(tables_len = tables.len()))]
    async fn extract_generic_infos_from_tables(
        &self,
        tables: Vec<Element>,
    ) -> Result<Vec<GenericTokenInfo>, DebankScraperError> {
        let mut generic_infos: Vec<Vec<(String, String)>> = Vec::new();

        let table_len = tables.len();
        for (index, table) in tables.into_iter().enumerate() {
            tracing::trace!("Processing table {}/{}", index + 1, table_len);
            let generic_info = self.extract_generic_infos_from_table(&table).await?;
            tracing::trace!(
                "Extracted {} generic infos from table {}/{}",
                generic_info.len(),
                index + 1,
                table_len
            );
            generic_infos.push(generic_info);
        }
        tracing::trace!("Finished processing tables");

        let generic_infos = self.parse_generic_info(generic_infos)?;
        Ok(generic_infos)
    }

    #[instrument(skip(self, table))]
    async fn extract_generic_infos_from_table(
        &self,
        table: &Element,
    ) -> Result<Vec<(String, String)>, DebankScraperError> {
        let mut generic_infos: Vec<(String, String)> = Vec::new();
        tracing::trace!("Locating tracking headers...");
        let tracking_headers = table
            .find_all(Locator::XPath("div[1]//span"))
            .await
            .change_context(DebankScraperError::ElementHtmlNotFound)?;
        tracing::trace!("Found {} headers", tracking_headers.len());

        tracing::trace!("Fetching header texts...");
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
        tracing::trace!("Fetched header texts");

        // Convert all results to strings returning an error if any of them fails
        tracing::trace!("Converting header texts to strings...");
        let headers = headers
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .change_context(DebankScraperError::GenericError)?;
        tracing::trace!("Headers: {:?}", headers);

        tracing::trace!("Locating tracking body...");
        let tracking_body = table
            .find(Locator::XPath("div[2]"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        tracing::trace!("Locating tracking rows...");
        let row_selector = "div.table_contentRow__Mi3k5.flex_flexRow__y0UR2";
        let rows = tracking_body
            .find_all(Locator::Css(row_selector))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;
        tracing::trace!("Found {} rows", rows.len());

        let row_len = rows.len();
        for (index, row) in rows.iter().enumerate() {
            tracing::trace!("Processing row {}/{}", index + 1, row_len);
            let zipped = self
                .extract_generic_infos_from_row(row, headers.clone())
                .await
                .change_context(DebankScraperError::ElementNotFound)?;

            generic_infos.extend(zipped);
        }

        tracing::trace!("Finished processing rows");
        Ok(generic_infos)
    }

    #[instrument(skip(self, row))]
    async fn extract_generic_infos_from_row(
        &self,
        row: &Element,
        headers: Vec<String>,
    ) -> Result<Vec<(String, String)>, DebankScraperError> {
        tracing::trace!("Locating row cells...");
        let cells = row
            .find_all(Locator::XPath("div"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;
        tracing::trace!("Found {} cells", cells.len());

        tracing::trace!("Fetching cell texts...");
        let values =
            futures::future::join_all(cells.iter().map(|cell| self.extract_cell_info(cell)))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

        if headers.len() != values.len() {
            bail!(DebankScraperError::HeadersValuesLengthMismatch);
        }

        let zipped = headers
            .clone()
            .into_iter()
            .zip(values.into_iter())
            .collect::<Vec<_>>();

        Ok(zipped)
    }

    #[instrument]
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
            _ => {
                tracing::error!("Unknown tracking type: {}", tracking_type);
                return Err(DebankScraperError::UnknownTrackingType)
                    .change_context(DebankScraperError::GenericError)?;
            }
        }
    }

    #[instrument]
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
                        tracing::error!("Unknown header: {}", header);
                        return Err(DebankScraperError::UnknownHeader.into());
                    }
                }
            }
            infos.push(info);
        }

        Ok(infos)
    }

    #[instrument]
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
            let (div_span_div_and_a_text, div_span_div_and_a_a) = join!(
                async {
                    div_span_div_and_a
                        .text()
                        .await
                        .change_context(DebankScraperError::ElementTextNotFound)
                },
                async {
                    div_span_div_and_a
                        .find(Locator::XPath("a"))
                        .await
                        .change_context(DebankScraperError::ElementNotFound)?
                        .text()
                        .await
                        .change_context(DebankScraperError::ElementTextNotFound)
                }
            );
            let div_span_div_and_a_text = div_span_div_and_a_text?;
            let div_span_div_and_a_a = div_span_div_and_a_a?;

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
    #[instrument]
    pub async fn access_profile(&self, user_id: &str) -> Result<(), DebankScraperError> {
        self.open_debank_url(user_id).await?;
        self.wait_data_updated().await?;
        Ok(())
    }

    #[instrument]
    pub async fn get_total_balance(&self) -> Result<f64, DebankScraperError> {
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
        event!(Level::DEBUG, balance_text = %balance_text, "Obtained balance text");

        format_balance(&balance_text)
            .map_err(|e| {
                tracing::error!("Failed to parse balance: {}", e);
                DebankScraperError::GenericError
            })
            .change_context(DebankScraperError::GenericError)
            .map(|balance| {
                event!(Level::DEBUG, balance = %balance, "Parsed balance");
                balance
            })
    }

    #[instrument]
    pub async fn explore_debank_profile(
        &self,
        user_id: &str,
    ) -> Result<HashMap<String, ChainInfo>, DebankScraperError> {
        let chain_summaries: Vec<Element> = self.locate_chain_summary_elements().await?;
        return self.get_chains_info(&chain_summaries).await;
    }

    #[instrument(skip(self, chain_summaries))]
    async fn get_chains_info(
        &self,
        chain_summaries: &[Element],
    ) -> Result<HashMap<String, ChainInfo>, DebankScraperError> {
        let mut chain_infos = HashMap::new();
        for (index, chain) in chain_summaries.iter().enumerate() {
            let span = tracing::span!(Level::DEBUG, "Chain", index = index);
            let chain_info = self.get_chain_info(chain).instrument(span).await?;
            chain_infos.insert(chain_info.name.clone(), chain_info);
        }
        Ok(chain_infos)
    }
}
