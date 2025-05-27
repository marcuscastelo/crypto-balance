use std::{collections::HashMap, fmt::Debug, time::Duration};
use thiserror::Error;

use error_stack::{bail, Result, ResultExt};
use fantoccini::{elements::Element, Locator};
use futures::join;
use reqwest::Url;
use tracing::{event, info, instrument, Instrument, Level};

use super::fantoccini_scraper_driver::FantocciniScraperDriver;
use crate::domain::debank::{
    ChainInfo, ChainProjectInfo, ChainWalletInfo, FarmingTokenInfo, LendingTokenInfo,
    LiquidityPoolTokenInfo, LockedTokenInfo, ProjectTracking, ProjectTrackingSection,
    RewardTokenInfo, SimpleTokenInfo, SpotTokenInfo, StakeTokenInfo, TokenInfo, VestingTokenInfo,
};

use crate::infrastructure::debank::balance::format_balance;

pub struct DebankBalanceScraper {
    driver: FantocciniScraperDriver,
}

impl Debug for DebankBalanceScraper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebankBalanceScraper").finish()
    }
}

#[derive(Error, Debug)]
pub enum DebankScraperError {
    #[error("Element not found")]
    ElementNotFound,
    #[error("Element text not found")]
    ElementTextNotFound,
    #[error("Headers and values length mismatch")]
    HeadersValuesLengthMismatch,
    #[error("Unknown header")]
    UnknownHeader,
    #[error("Unknown tracking type")]
    UnknownTrackingType,
    #[error("Failed to parse URL")]
    UrlParseError,
    #[error("Failed to create driver")]
    FailedToCreateDriver,
    #[error("Failed to navigate to URL")]
    FailedToNavigateToUrl,
    #[error("Failed to get chain info")]
    FailedToGetChainInfo,
    #[error("Failed to explore tracking")]
    FailedToExploreTracking,
    #[error("Failed to extract cell info")]
    FailedToExtractCellInfo,
    #[error("Failed to click element")]
    FailedToClickElement,
    #[error("Generic error")]
    GenericError,
}

impl DebankBalanceScraper {
    #[instrument]
    pub async fn new() -> Result<Self, DebankScraperError> {
        let driver = FantocciniScraperDriver::new()
            .await
            .change_context(DebankScraperError::FailedToCreateDriver)?;
        Ok(Self { driver })
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self, chain))]
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

        tokio::time::sleep(Duration::from_millis(100))
            .instrument(tracing::span!(Level::DEBUG, "sleep_after_click"))
            .await;

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

    #[instrument(skip(self, wallet, projects))]
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

    #[instrument(skip(self, wallet))]
    async fn get_chain_wallet_info(
        &self,
        wallet: &Element,
    ) -> Result<ChainWalletInfo, DebankScraperError> {
        let usd_value = wallet
            .find(Locator::XPath("div[1]/div[2]"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
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

    #[instrument(skip(self, row))]
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

    #[instrument(skip(self, row))]
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

    #[instrument(skip(self, project))]
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
                    .change_context(DebankScraperError::ElementNotFound);

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

            let html = project.html(false).await;
            name.ok_or(DebankScraperError::ElementTextNotFound)
                .attach_printable_lazy(|| {
                    format!(
                        "Failed to get project name after 3 retries: html={:?}",
                        html
                    )
                })?
        };

        let tracking_elements = project
            .find_all(Locator::Css("div.Panel_container__Vltd1"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .into_iter()
            .map(|element| (name.as_ref(), element))
            .collect::<Vec<_>>();

        let tracking_futures = tracking_elements
            .iter()
            .map(|(name, element)| self.explore_tracking(name, element));

        let trackings = futures::future::join_all(tracking_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .change_context(DebankScraperError::FailedToExploreTracking)?;

        Ok(ChainProjectInfo { name, trackings })
    }

    #[instrument(skip(self, tracking_vltd1_element))]
    async fn explore_tracking(
        &self,
        project_name: &str,
        tracking_vltd1_element: &Element,
    ) -> Result<ProjectTracking, DebankScraperError> {
        tracing::info!("Fetching tracking type for project: {project_name}");
        let tracking_type = tracking_vltd1_element
            .find(Locator::XPath("div[1]/div[1]/div[1]"))
            .instrument(tracing::span!(Level::DEBUG, "tracking_type"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?
            .text()
            .await
            .change_context(DebankScraperError::ElementTextNotFound)?;

        tracing::info!(
            "Tracking type for project {}: {}",
            project_name,
            tracking_type
        );

        tracing::info!(
            "Locating tracking tables for project/type: {}/{}",
            project_name,
            tracking_type
        );
        let tables = tracking_vltd1_element
            .find_all(Locator::XPath("div[2]/div"))
            .instrument(tracing::span!(Level::DEBUG, "tracking_tables"))
            .await
            .change_context(DebankScraperError::ElementNotFound)?;

        tracing::info!(
            "Found {} tables for project/type: {}/{}",
            tables.len(),
            project_name,
            tracking_type
        );

        let generic_infos = self
            .extract_generic_infos_from_tables(project_name, &tracking_type, tables)
            .await?;

        tracing::debug!(
            "Extracted {} generic infos for project/type: {}/{} -> {:?}",
            generic_infos.len(),
            project_name,
            tracking_type,
            generic_infos
        );

        let specialized =
            self.specialize_generic_info(project_name, &tracking_type, generic_infos)?;

        tracing::info!(
            "Specialized tracking for project/type: {}/{} -> {:?}",
            project_name,
            tracking_type,
            specialized,
        );

        Ok(specialized)
    }

    #[instrument(skip(self, tables), fields(tables_len = tables.len()))]
    async fn extract_generic_infos_from_tables(
        &self,
        project_name: &str,
        tracking_type: &str,
        tables: Vec<Element>,
    ) -> Result<Vec<TokenInfo>, DebankScraperError> {
        let mut generic_infos: Vec<Vec<(String, String)>> = Vec::new();

        let table_len = tables.len();
        for (index, table) in tables.into_iter().enumerate() {
            tracing::trace!("Processing table {}/{}", index + 1, table_len);
            let generic_info = self
                .extract_generic_infos_from_table(project_name, tracking_type, &table)
                .await?;
            tracing::trace!(
                "Extracted {} generic infos from table {}/{}: {:?}",
                generic_info.len(),
                index + 1,
                table_len,
                generic_infos
            );
            generic_infos.extend(generic_info);
        }
        tracing::info!(
            "Finished processing tables for project/type: {}/{} -> {:?}",
            project_name,
            tracking_type,
            generic_infos
        );

        let generic_infos = self.parse_generic_info(generic_infos)?;

        tracing::info!(
            "Parsed {} generic infos for project/type: {}/{} -> {:?}",
            generic_infos.len(),
            project_name,
            tracking_type,
            generic_infos
        );
        Ok(generic_infos)
    }

    #[instrument(skip(self, table))]
    async fn extract_generic_infos_from_table(
        &self,
        project_name: &str,
        tracking_type: &str,
        table: &Element,
    ) -> Result<Vec<Vec<(String, String)>>, DebankScraperError> {
        let mut generic_infos: Vec<Vec<(String, String)>> = Vec::new();
        tracing::trace!("Locating tracking headers...");
        // table_header__onfbK
        let html = table.html(false).await;
        let tracking_headers = table
            .find_all(Locator::XPath("div[1]//span"))
            .await
            .change_context(DebankScraperError::ElementNotFound)
            .attach_printable_lazy(|| {
                let msg = format!("Failed to find tracking headers in table for project: {project_name}, type: {tracking_type}, html: {html:?}");
                tracing::error!("{}", msg);
                msg
            })?;
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
            tracing::trace!(
                "Processing row {}/{} of {}/{}",
                index + 1,
                row_len,
                project_name,
                tracking_type
            );
            let row_key_values = self
                .extract_generic_infos_from_row(row, headers.clone())
                .await
                .change_context(DebankScraperError::ElementNotFound)?;

            tracing::info!(
                "Processed row {}/{} of {}/{} -> {:?}",
                index + 1,
                row_len,
                project_name,
                tracking_type,
                row_key_values,
            );

            generic_infos.push(row_key_values);
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

    #[instrument(skip(self))]
    fn specialize_generic_info(
        &self,
        project_name: &str,
        tracking_type: &str,
        generic_infos: Vec<TokenInfo>,
    ) -> Result<ProjectTracking, DebankScraperError> {
        // Remove ($XX.XX) from rewards, if present
        let regex = regex::Regex::new(r"\(<?\$[0-9,.]+\)")
            .change_context(DebankScraperError::GenericError)?;

        let generic_infos = generic_infos
            .into_iter()
            .map(|mut generic| {
                if let Some(rewards) = &generic.rewards {
                    tracing::trace!("Replacing rewards dollar representation in generic info");
                    tracing::trace!("Original rewards: {:?}", rewards);
                    generic.rewards = Some(regex.replace_all(rewards, "").trim().to_string());
                    tracing::trace!("Updated rewards: {:?}", generic.rewards);
                }
                generic
            })
            .collect::<Vec<_>>();

        match tracking_type {
            "Yield" => Ok(ProjectTracking::YieldFarm {
                yield_farm: generic_infos
                    .into_iter()
                    .map(|generic| SimpleTokenInfo {
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
                    .map(|generic| SimpleTokenInfo {
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
                token_sections: vec![ProjectTrackingSection {
                    title: None,
                    tokens: generic_infos
                        .into_iter()
                        .map(|generic| FarmingTokenInfo {
                            token_name: generic.token_name,
                            pool: generic.pool.expect("Pool not found"),
                            balance: generic.balance.expect("Balance not found"),
                            usd_value: generic.usd_value.expect("USD value not found"),
                            rewards: generic.rewards,
                        })
                        .collect(),
                }],
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

    #[instrument(skip(self))]
    fn parse_generic_info(
        &self,
        generic_info: Vec<Vec<(String, String)>>,
    ) -> Result<Vec<TokenInfo>, DebankScraperError> {
        let mut infos = Vec::new();
        tracing::trace!("Parsing {} generic infos", generic_info.len());
        for row_values in generic_info.as_slice() {
            tracing::trace!("Parsing row: {:?}", row_values);
            let mut info = TokenInfo::default();
            for (header, value) in row_values.as_slice() {
                tracing::trace!("Info before: {:?}", info);
                tracing::trace!("Header: {}, Value: {}", header, value);
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
                        if info.token_name.is_none() {
                            info.token_name = value.clone().into();
                        }
                        if header == "Rewards" {
                            info.rewards = value.clone().into();
                        }
                    }
                    _ => {
                        tracing::error!("Unknown header: {}", header);
                        return Err(DebankScraperError::UnknownHeader.into());
                    }
                }
                tracing::trace!("Info after: {:?}", info);
            }
            if info.token_name.is_none() {
                info.token_name = info.pool.clone();
            }
            infos.push(info);
        }

        Ok(infos)
    }

    #[instrument(skip(self, cell))]
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
    #[instrument(skip(self))]
    pub async fn access_profile(&self, user_id: &str) -> Result<(), DebankScraperError> {
        self.open_debank_url(user_id).await?;
        self.wait_data_updated().await?;
        Ok(())
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
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
