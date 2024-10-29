use std::collections::HashMap;

pub struct CoinGeckoApi;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CoinResponse {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CoinListResponse(pub Vec<CoinResponse>);

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
        let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
        let prices: PricesResponse = serde_json::from_str(&response).unwrap();
        prices
    }
}
