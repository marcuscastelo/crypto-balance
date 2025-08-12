use crate::adapters::price::price::get_token_prices;
use crate::adapters::sheets::spreadsheet_manager::SpreadsheetManager;
use crate::adapters::sheets::spreadsheet_read::SpreadsheetRead;
use crate::adapters::sheets::spreadsheet_write::SpreadsheetWrite;
use crate::domain::routine::{Routine, RoutineError};
use crate::domain::sheets::ranges;
use error_stack::{report, ResultExt};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

use tracing::instrument;

#[derive(Error, Debug)]
enum TokenPricesRoutineError {
    #[error("failed execute spreadsheet operation")]
    SpreadsheetError,
    #[error("spreadsheet data is invalid: {details}")]
    InvalidDataError { details: &'static str },
}

#[derive(Debug)]
pub struct TokenPricesRoutine {
    pub spreadsheet_manager: Arc<SpreadsheetManager>,
}

impl TokenPricesRoutine {
    pub fn new(spreadsheet_manager: Arc<SpreadsheetManager>) -> Self {
        Self {
            spreadsheet_manager,
        }
    }

    #[instrument]
    async fn get_token_ids_from_spreadsheet(
        &self,
    ) -> error_stack::Result<Vec<String>, TokenPricesRoutineError> {
        let token_ids = self
            .spreadsheet_manager
            .read_named_range(ranges::tokens::RO_IDS)
            .await
            .change_context(TokenPricesRoutineError::SpreadsheetError)?;

        Ok(token_ids)
    }

    #[instrument]
    async fn get_current_prices_from_spreadsheet(
        &self,
    ) -> error_stack::Result<Vec<f64>, TokenPricesRoutineError> {
        let current_prices = self
            .spreadsheet_manager
            .read_named_range(ranges::tokens::RW_PRICES)
            .await
            .change_context(TokenPricesRoutineError::SpreadsheetError)?
            .into_iter()
            .map(|x| {
                x.replace(['$', ','], "")
                    .parse::<f64>()
                    .change_context(TokenPricesRoutineError::InvalidDataError {
                        details: "Failed to parse price from spreadsheet",
                    })
                    .attach_printable_lazy(|| format!("Failed to parse price: {}", x))
            })
            .collect::<error_stack::Result<_, _>>()?;

        Ok(current_prices)
    }

    #[instrument]
    fn order_prices(
        &self,
        tokens: &Vec<String>,
        prices: &HashMap<String, Option<f64>>,
        fallback_prices: Vec<f64>,
    ) -> error_stack::Result<Vec<f64>, TokenPricesRoutineError> {
        let ordered_prices = tokens
            .iter()
            .enumerate()
            .map(|(i, token)| match prices.get(token) {
                Some(price) => price.ok_or(report!(TokenPricesRoutineError::InvalidDataError {
                    details: "Price for token is None",
                })),
                None => Ok(fallback_prices.get(i).copied().unwrap_or(0.0)),
            })
            .collect::<error_stack::Result<_, _>>()?;

        Ok(ordered_prices)
    }

    #[instrument]
    async fn update_prices_on_spreadsheet(
        &self,
        new_prices: Vec<f64>,
    ) -> error_stack::Result<(), TokenPricesRoutineError> {
        let values = new_prices
            .iter()
            .map(|x| format!("${}", x))
            .collect::<Vec<_>>();
        self.spreadsheet_manager
            .write_named_column(ranges::tokens::RW_PRICES, values.as_ref())
            .await
            .change_context(TokenPricesRoutineError::SpreadsheetError)?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Routine for TokenPricesRoutine {
    fn name(&self) -> &'static str {
        "TokenPricesRoutine"
    }

    #[instrument(skip(self), name = "TokenPricesRoutine::run")]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        tracing::info!("Running TokenPricesRoutine");

        tracing::info!("Prices: 📋 Listing all tokens in the spreadsheet");
        let tokens = self.get_token_ids_from_spreadsheet().await.change_context(
            RoutineError::routine_failure("Failed to get token ids from spreadsheet"),
        )?;

        tracing::info!("Prices: ☁️  Getting prices of all tokens from Coingecko");
        let prices = get_token_prices(tokens.as_ref()).await;

        tracing::info!("Prices: 📝 Reading the current prices from the spreadsheet");
        let spreadsheet_prices = self
            .get_current_prices_from_spreadsheet()
            .await
            .change_context(RoutineError::routine_failure(
                "Failed to get current prices from spreadsheet",
            ))?;

        tracing::info!("Prices: 📝 Updating the prices on the spreadsheet");
        let new_prices = self
            .order_prices(&tokens, &prices, spreadsheet_prices)
            .change_context(RoutineError::routine_failure("Failed to order prices"))?;

        self.update_prices_on_spreadsheet(new_prices)
            .await
            .change_context(RoutineError::routine_failure(
                "Failed to update prices on spreadsheet",
            ))?;

        tracing::info!("Prices: ✅ Updated token prices on the spreadsheet");

        Ok(())
    }
}
