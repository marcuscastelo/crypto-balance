use crate::prelude::*;
use ::binance::api::Binance;
use ::binance::market::Market;
use ::binance::rest_model::Prices;
use google_sheets4::api::ValueRange;
use serde_json::Value;
use std::collections::HashMap;

pub struct UpdateTokenPricesOnSheetsViaBinanceRoutine;

#[async_trait::async_trait]
impl Routine for UpdateTokenPricesOnSheetsViaBinanceRoutine {
    async fn run(&self) {
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let token_names: Vec<String> = spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into();

        let normal_pairs = token_names
            .clone()
            .into_iter()
            .map(|name| (name.clone(), format!("{}USDT", name.clone())))
            .collect::<Vec<(String, String)>>();

        let reverse_pairs = token_names
            .clone()
            .into_iter()
            .map(|name| (name.clone(), format!("USDT{}", name.clone())))
            .collect::<Vec<(String, String)>>();

        let binance_market: Market = Binance::new(None, None);
        let Prices::AllPrices(all_prices) = binance_market
            .get_all_prices()
            .await
            .expect("Should get all prices from Binance API");

        let mut prices: HashMap<_, _> =
            HashMap::with_capacity(normal_pairs.len() + reverse_pairs.len());

        // TODO: Move USDT mentions to a constant for easier maintenance
        // Special case for USDT/USDT pair that doesn't exist since it wouldn't make sense
        prices.insert("USDT".to_string(), 1.0);

        for symbol_price in all_prices {
            if let Some(normal_pair) = normal_pairs.iter().find(|x| x.1 == symbol_price.symbol) {
                prices.insert(normal_pair.0.clone(), symbol_price.price);
            }

            if let Some(reverse_pair) = reverse_pairs.iter().find(|x| x.1 == symbol_price.symbol) {
                prices.insert(reverse_pair.0.clone(), 1.0 / symbol_price.price);
            }
        }

        let mut token_prices_in_order = Vec::with_capacity(token_names.len());
        for token_name in &token_names {
            token_prices_in_order.push(prices.get(token_name).unwrap_or_else(|| {
                println!("Warning: No price for {}", token_name);
                &0.0
            }));
        }

        println!(
            "{:?}",
            token_names
                .iter()
                .zip(token_prices_in_order.clone())
                .collect::<Vec<_>>()
        );

        let a = ValueRange {
            range: None,
            major_dimension: None,
            values: Some(
                token_prices_in_order
                    .into_iter()
                    .map(|price| vec![Value::Number(serde_json::Number::from_f64(*price).unwrap())])
                    .collect::<Vec<_>>(),
            ),
        };

        spreadsheet_manager
            .write_named_range(ranges::tokens::RW_PRICES, a)
            .await
            .expect("Should write prices to the spreadsheet");
    }
}
