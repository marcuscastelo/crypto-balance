use std::collections::HashMap;

use crate::script::sonar_script::sonar;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SonarResponse {
    pub elements: Option<Vec<SonarElement>>,
    pub value: Option<f64>,
    pub tokenInfo: Option<TokenInfoByNetwork>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub statusCode: Option<u16>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenInfoByNetwork {
    pub solana: HashMap<String, TokenInfo>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenInfo {
    pub networkId: String,
    pub address: String,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub extensions: Option<TokenExtensions>,
    pub logoURI: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TokenExtensions {
    pub coingeckoId: Option<String>,
    pub showAsToken: Option<bool>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SonarElement {
    #[serde(rename = "type")]
    pub elementType: String,
    pub networkId: String,
    pub platformId: String,
    pub label: String,
    pub value: Option<f64>,
    pub data: ElementData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ElementData {
    pub assets: Option<Vec<Asset>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Asset {
    #[serde(rename = "type")]
    pub assetType: String,
    pub networkId: String,
    pub value: Option<f64>,
    pub data: AssetData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AssetData {
    pub address: String,
    pub amount: Option<f64>,
    pub price: Option<f64>,
}

pub struct SonarWatchScraper;

impl SonarWatchScraper {
    pub async fn scrape(&self) -> anyhow::Result<SonarResponse> {
        let resp = sonar().await;

        let sonar_response: SonarResponse = serde_json::from_str(&resp)
            .map_err(|error| anyhow::anyhow!("Failed to parse sonar response: {}", error))?;

        if let Some(status_code) = sonar_response.statusCode {
            if status_code != 200 {
                return Err(anyhow::anyhow!(
                    "Failed to scrape SonarWatch balance: {} - {}",
                    sonar_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_owned()),
                    sonar_response
                        .message
                        .unwrap_or_else(|| "No message".to_owned())
                ));
            }
        }

        Ok(sonar_response)
    }
}
