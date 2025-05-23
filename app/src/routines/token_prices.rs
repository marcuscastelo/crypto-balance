use std::collections::HashMap;

use crate::{
    price::domain::price::get_token_prices,
    sheets::{data::spreadsheet_manager::SpreadsheetManager, into::MyInto, ranges},
};
use tracing::instrument;

use super::routine::{Routine, RoutineError};

#[derive(Debug)]
pub struct TokenPricesRoutine<'s> {
    pub spreadsheet_manager: &'s SpreadsheetManager,
}

impl<'s> TokenPricesRoutine<'s> {
    pub fn new(spreadsheet_manager: &'s SpreadsheetManager) -> Self {
        Self {
            spreadsheet_manager,
        }
    }

    #[instrument]
    async fn get_token_ids_from_spreadsheet(&self) -> Vec<String> {
        self.spreadsheet_manager
            .read_named_range(ranges::tokens::RO_IDS)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }

    #[instrument]
    async fn get_current_prices_from_spreadsheet(&self) -> Vec<f64> {
        self.spreadsheet_manager
            .read_named_range(ranges::tokens::RW_PRICES)
            .await
            .expect("Should have content")
            .values
            .unwrap_or(vec![])
            .my_into()
            .into_iter()
            .map(|x| {
                x.replace(['$', ','], "")
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Should be a number: {}", x))
            })
            .collect::<Vec<_>>()
    }

    #[instrument]
    fn order_prices(
        &self,
        tokens: &Vec<String>,
        prices: &HashMap<String, Option<f64>>,
        fallback_prices: Vec<f64>,
    ) -> Vec<f64> {
        tokens
            .iter()
            .enumerate()
            .map(|(i, token)| match prices.get(token) {
                Some(price) => {
                    price.expect("Should have a price since token was found on CoinGecko")
                }
                None => fallback_prices.get(i).copied().unwrap_or(0.0),
            })
            .collect()
    }

    #[instrument]
    async fn update_prices_on_spreadsheet(&self, new_prices: Vec<f64>) {
        let values = new_prices
            .iter()
            .map(|x| format!("${}", x))
            .collect::<Vec<_>>();
        self.spreadsheet_manager
            .write_named_column(ranges::tokens::RW_PRICES, values.as_ref())
            .await
            .expect("Should have written successfully");
    }
}

#[async_trait::async_trait]
impl<'s> Routine for TokenPricesRoutine<'s> {
    fn name(&self) -> &'static str {
        "TokenPricesRoutine"
    }

    #[instrument(skip(self), name = "TokenPricesRoutine::run")]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("Running TokenPricesRoutine");

        tracing::info!("Prices: 📋 Listing all tokens in the spreadsheet");
        let tokens = self.get_token_ids_from_spreadsheet().await;

        tracing::info!("Prices: ☁️  Getting prices of all tokens from Coingecko");
        let prices = get_token_prices(tokens.as_ref()).await;

        tracing::info!("Prices: 📝 Reading the current prices from the spreadsheet");
        let spreadsheet_prices = self.get_current_prices_from_spreadsheet().await;

        tracing::info!("Prices: 📝 Updating the prices on the spreadsheet");
        let new_prices = self.order_prices(&tokens, &prices, spreadsheet_prices);
        self.update_prices_on_spreadsheet(new_prices).await;

        tracing::info!("Prices: ✅ Updated token prices on the spreadsheet");

        Ok(())
    }
}
