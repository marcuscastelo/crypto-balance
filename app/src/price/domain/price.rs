use std::collections::HashMap;

use crate::price::data::coingecko::api::CoinGeckoApi;

pub async fn get_token_prices(tokens: &[String]) -> HashMap<String, Option<f64>> {
    CoinGeckoApi
        .prices(tokens)
        .await
        .0
        .into_iter()
        .map(|(k, v)| (k, v.usd))
        .collect()
}
