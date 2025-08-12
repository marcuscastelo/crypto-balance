use std::time::Duration;

use chrono::{DateTime, Utc};
use error_stack::ResultExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::domain::debank::{Chain, DebankResponse};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_JOB_WAIT_TIME: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("HTTP request failed")]
    HttpError,

    #[error("HTTP status error: {0}")]
    HttpStatusError(String),

    #[error("JSON parsing failed")]
    JsonError,

    #[error("Scrape job failed: {0}")]
    JobFailed(String),

    #[error("Scrape job timed out after {0} seconds")]
    JobTimeout(u64),

    #[error("Invalid job status: {0}")]
    InvalidJobStatus(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeRequest {
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(default = "default_true")]
    pub save_html: bool,
    #[serde(default = "default_true")]
    pub save_screenshot: bool,
    #[serde(default = "default_true")]
    pub headless: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub message: String,
    // pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub progress: Option<String>,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResultResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub data: Option<PortfolioData>,
    pub error_message: Option<String>,
    // pub created_at: DateTime<Utc>,
    // pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

// API Portfolio structures - these mirror the DebankResponse structures but use the API naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioData {
    pub total_usd_value: Option<String>,
    #[serde(default)]
    pub chains: Vec<ChainInfo>,
    pub metadata: Option<PortfolioMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetadata {
    pub wallet_address: String,
    #[serde(default)]
    pub chain_filter: String,
    pub url: String,
    pub screenshot_path: Option<String>,
    pub html_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub name: String,
    pub wallet_info: Option<WalletInfo>,
    #[serde(default)]
    pub project_info: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub usd_value: Option<String>,
    #[serde(default)]
    pub tokens: Vec<SpotTokenInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub trackings: Vec<Tracking>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracking {
    pub tracking_type: String,
    #[serde(default)]
    pub token_sections: Vec<TokenSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSection {
    pub title: String,
    #[serde(default)]
    pub tokens: Vec<TokenInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_name: Option<String>,
    pub pool: Option<String>,
    #[serde(default)]
    pub balance: String,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub claimable_amount: Option<String>,
    pub end_time: Option<String>,
    pub usd_value: Option<String>,
    pub variant_header: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTokenInfo {
    pub name: String,
    pub price: String,
    pub amount: String,
    pub usd_value: Option<String>,
}

// Conversion implementations to map API structures to domain structures
impl From<PortfolioData> for DebankResponse {
    fn from(portfolio: PortfolioData) -> Self {
        Self {
            total_usd_value: portfolio.total_usd_value.unwrap_or_default(),
            chains: portfolio.chains.into_iter().map(Chain::from).collect(),
            metadata: portfolio.metadata.map(|m| m.into()),
        }
    }
}

impl From<PortfolioMetadata> for crate::domain::debank::DebankMetadata {
    fn from(metadata: PortfolioMetadata) -> Self {
        Self {
            wallet_address: metadata.wallet_address,
            chain_filter: metadata.chain_filter,
            url: metadata.url,
            screenshot_path: metadata.screenshot_path.unwrap_or_default(),
            html_path: metadata.html_path.unwrap_or_default(),
        }
    }
}

impl From<ChainInfo> for Chain {
    fn from(chain: ChainInfo) -> Self {
        Self {
            name: chain.name,
            wallet_info: chain.wallet_info.map(|w| w.into()),
            project_info: chain.project_info.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<WalletInfo> for crate::domain::debank::ChainWallet {
    fn from(wallet: WalletInfo) -> Self {
        Self {
            usd_value: wallet.usd_value.unwrap_or_default(),
            tokens: wallet.tokens.into_iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<Project> for crate::domain::debank::Project {
    fn from(project: Project) -> Self {
        Self {
            name: project.name,
            trackings: project.trackings.into_iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<Tracking> for crate::domain::debank::ProjectTracking {
    fn from(tracking: Tracking) -> Self {
        Self {
            tracking_type: tracking.tracking_type,
            token_sections: tracking
                .token_sections
                .into_iter()
                .map(|s| s.into())
                .collect(),
        }
    }
}

impl From<TokenSection> for crate::domain::debank::ProjectTrackingSection {
    fn from(section: TokenSection) -> Self {
        Self {
            title: section.title,
            tokens: section.tokens.into_iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<TokenInfo> for crate::domain::debank::TokenInfo {
    fn from(token: TokenInfo) -> Self {
        Self {
            token_name: token.token_name,
            pool: token.pool,
            balance: Some(token.balance),
            rewards: token.rewards,
            unlock_time: token.unlock_time,
            claimable_amount: token.claimable_amount,
            end_time: token.end_time,
            usd_value: token.usd_value,
            variant_header: token.variant_header,
        }
    }
}

impl From<SpotTokenInfo> for crate::domain::debank::SpotTokenInfo {
    fn from(token: SpotTokenInfo) -> Self {
        Self {
            name: token.name,
            price: token.price,
            amount: token.amount,
            usd_value: token.usd_value.unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub struct DebankApiClient {
    client: Client,
    base_url: String,
}

impl DebankApiClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    #[instrument(skip(self))]
    pub async fn scrape_wallet(
        &self,
        request: ScrapeRequest,
    ) -> error_stack::Result<DebankResponse, ApiClientError> {
        event!(Level::INFO, wallet = %request.wallet_address, "Starting wallet scrape");

        // Step 1: Create scrape job
        let job_response = self.create_scrape_job(request).await?;
        event!(Level::INFO, job_id = %job_response.job_id, "Scrape job created");

        // Step 2: Poll job status until completion
        let result = self.wait_for_job_completion(&job_response.job_id).await?;

        // Step 3: Get job result
        match result.data {
            Some(portfolio_data) => {
                event!(Level::INFO, job_id = %job_response.job_id, "Scrape job completed successfully");
                Ok(portfolio_data.into())
            }
            None => {
                let error_msg = result
                    .error_message
                    .unwrap_or_else(|| "No data returned from job".to_string());
                Err(ApiClientError::JobFailed(error_msg).into())
            }
        }
    }

    #[instrument(skip(self))]
    async fn create_scrape_job(
        &self,
        request: ScrapeRequest,
    ) -> error_stack::Result<ScrapeResponse, ApiClientError> {
        let url = format!("{}/api/scrape", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .change_context(ApiClientError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ApiClientError::HttpStatusError(format!(
                "HTTP error {}: {}",
                status, error_text
            ))
            .into());
        }

        let text = response.text().await.unwrap();

        tracing::debug!(response = text, "Response OK 200");

        let scrape_response: ScrapeResponse =
            serde_json::from_str(text.as_str()).change_context(ApiClientError::JsonError)?;

        Ok(scrape_response)
    }

    #[instrument(skip(self))]
    async fn wait_for_job_completion(
        &self,
        job_id: &str,
    ) -> error_stack::Result<JobResultResponse, ApiClientError> {
        let start_time = std::time::Instant::now();

        loop {
            // Check if we've exceeded the maximum wait time
            if start_time.elapsed() > MAX_JOB_WAIT_TIME {
                return Err(ApiClientError::JobTimeout(MAX_JOB_WAIT_TIME.as_secs()).into());
            }

            // Get job status
            let status = self.get_job_status(job_id).await?;

            match status.status {
                JobStatus::Completed => {
                    event!(Level::INFO, job_id = %job_id, "Job completed");
                    return self.get_job_result(job_id).await;
                }
                JobStatus::Failed => {
                    let error_msg = status
                        .error_message
                        .unwrap_or_else(|| "Job failed with unknown error".to_string());
                    return Err(ApiClientError::JobFailed(error_msg).into());
                }
                JobStatus::Pending | JobStatus::InProgress => {
                    if let Some(progress) = &status.progress {
                        event!(Level::DEBUG, job_id = %job_id, progress = %progress, "Job in progress");
                    }

                    // Wait before polling again
                    tokio::time::sleep(JOB_POLL_INTERVAL).await;
                }
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_job_status(
        &self,
        job_id: &str,
    ) -> error_stack::Result<JobStatusResponse, ApiClientError> {
        let url = format!("{}/api/jobs/{}", self.base_url, job_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .change_context(ApiClientError::HttpError)?;

        if !response.status().is_success() {
            return Err(ApiClientError::HttpStatusError(format!(
                "HTTP error {}",
                response.status()
            ))
            .into());
        }

        let text = response.text().await.unwrap();

        tracing::debug!(response = text, "Response OK 200");

        let status_response: JobStatusResponse =
            serde_json::from_str(text.as_str()).change_context(ApiClientError::JsonError)?;

        Ok(status_response)
    }

    #[instrument(skip(self))]
    async fn get_job_result(
        &self,
        job_id: &str,
    ) -> error_stack::Result<JobResultResponse, ApiClientError> {
        let url = format!("{}/api/results/{}", self.base_url, job_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .change_context(ApiClientError::HttpError)?;

        if !response.status().is_success() {
            return Err(ApiClientError::HttpStatusError(format!(
                "HTTP error {}",
                response.status()
            ))
            .into());
        }

        let result_response: JobResultResponse = response
            .json()
            .await
            .change_context(ApiClientError::JsonError)?;

        Ok(result_response)
    }
}
