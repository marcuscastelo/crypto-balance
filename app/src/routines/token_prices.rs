use std::collections::HashMap;

use crate::{
    price::domain::price::get_token_prices,
    sheets::{data::spreadsheet_manager::SpreadsheetManager, into::MyInto, ranges},
};
use tracing::instrument;

use super::routine::{Routine, RoutineError};

#[derive(Debug)]
pub struct TokenPricesRoutine;

impl TokenPricesRoutine {
    #[instrument]
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }

    #[instrument]
    async fn get_token_ids_from_spreadsheet(
        &self,
        spreadsheet_manager: &SpreadsheetManager,
    ) -> Vec<String> {
        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_IDS)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }

    #[instrument]
    async fn get_current_prices_from_spreadsheet(
        &self,
        spreadsheet_manager: &SpreadsheetManager,
    ) -> Vec<f64> {
        spreadsheet_manager
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
    async fn update_prices_on_spreadsheet(
        &self,
        spreadsheet_manager: &SpreadsheetManager,
        new_prices: Vec<f64>,
    ) {
        let values = new_prices
            .iter()
            .map(|x| format!("${}", x))
            .collect::<Vec<_>>();
        spreadsheet_manager
            .write_named_column(ranges::tokens::RW_PRICES, values.as_ref())
            .await
            .expect("Should have written successfully");
    }
}

#[async_trait::async_trait]
impl Routine for TokenPricesRoutine {
    fn name(&self) -> &'static str {
        "TokenPricesRoutine"
    }

    #[instrument(skip(self))]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("Running TokenPricesRoutine");

        tracing::info!("Prices: Creating SpreadsheetManager instance");
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        tracing::info!("Prices: 📋 Listing all tokens in the spreadsheet");
        let tokens = self
            .get_token_ids_from_spreadsheet(&spreadsheet_manager)
            .await;

        tracing::info!("Prices: ☁️  Getting prices of all tokens from Coingecko");
        let prices = get_token_prices(tokens.as_ref()).await;

        tracing::info!("Prices: 📝 Reading the current prices from the spreadsheet");
        let spreadsheet_prices = self
            .get_current_prices_from_spreadsheet(&spreadsheet_manager)
            .await;

        tracing::info!("Prices: 📝 Updating the prices on the spreadsheet");
        let new_prices = self.order_prices(&tokens, &prices, spreadsheet_prices);
        self.update_prices_on_spreadsheet(&spreadsheet_manager, new_prices)
            .await;

        tracing::info!("Prices: ✅ Updated token prices on the spreadsheet");

        Ok(())
    }
}
