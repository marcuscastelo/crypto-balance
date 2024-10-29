use std::collections::HashMap;

use crate::script::sonar_script::sonar;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct SonarResponse {
    elements: Option<Vec<SonarElement>>,
    value: Option<f64>,
    tokenInfo: Option<TokenInfoByNetwork>,
    message: Option<String>,
    error: Option<String>,
    statusCode: Option<u16>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct TokenInfoByNetwork {
    solana: HashMap<String, TokenInfo>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct TokenInfo {
    networkId: String,
    address: String,
    decimals: u8,
    name: String,
    symbol: String,
    extensions: Option<TokenExtensions>,
    logoURI: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct TokenExtensions {
    coingeckoId: Option<String>,
    showAsToken: Option<bool>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct SonarElement {
    #[serde(rename = "type")]
    elementType: String,
    networkId: String,
    platformId: String,
    label: String,
    value: Option<f64>,
    data: ElementData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct ElementData {
    assets: Option<Vec<Asset>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct Asset {
    #[serde(rename = "type")]
    assetType: String,
    networkId: String,
    value: Option<f64>,
    data: AssetData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct AssetData {
    address: String,
    amount: Option<f64>,
    price: Option<f64>,
}

pub async fn scrape() -> anyhow::Result<f64> {
    // let address = "3yy1dGAXHDRqGRwaVP3GfEpvSqDs251EiKN79NquewaR";

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

    if let Some(value) = sonar_response.value {
        Ok(value)
    } else {
        Err(anyhow::anyhow!(
            "Failed to scrape SonarWatch: value field not present in response"
        ))
    }
}
