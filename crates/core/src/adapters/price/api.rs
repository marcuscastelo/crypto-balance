use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
use std::collections::HashMap;

pub struct CoinGeckoApi;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PriceResponse {
    pub usd: Option<f64>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PricesResponse(pub HashMap<String, PriceResponse>);

impl CoinGeckoApi {
    pub async fn prices(&self, tokens: &[String]) -> PricesResponse {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            tokens.join(",")
        );
        tracing::trace!("CoinGecko API request URL: {}", url);

        // Build headers similar to the provided curl example. The most important
        // header is the User-Agent; we also set Accept and Accept-Language.
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:145.0) Gecko/20100101 Firefox/145.0",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        tracing::trace!("CoinGecko API response: {}", response);
        let prices: PricesResponse = serde_json::from_str(&response).unwrap();
        tracing::trace!("Parsed CoinGecko API response: {:?}", prices);
        prices
    }
}
