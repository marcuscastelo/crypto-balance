use std::collections::HashMap;

use crate::{
    cli::progress::{finish_progress, new_progress, ProgressBarExt},
    prelude::*,
    price::domain::price::get_token_prices,
};
use google_sheets4::api::ValueRange;
use indicatif::ProgressBar;
use into::MyInto;
use spreadsheet_manager::SpreadsheetManager;
use value_range_factory::ValueRangeFactory;

use super::routine::{Routine, RoutineResult};

pub struct TokenPricesRoutine;

impl TokenPricesRoutine {
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }

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

    async fn update_prices_on_spreadsheet(
        &self,
        spreadsheet_manager: &SpreadsheetManager,
        new_prices: Vec<f64>,
    ) {
        let values = new_prices
            .iter()
            .map(|x| format!("${:.2}", x))
            .collect::<Vec<_>>();
        spreadsheet_manager
            .write_named_range(
                ranges::tokens::RW_PRICES,
                ValueRange::from_rows(values.as_ref()),
            )
            .await
            .expect("Should have written successfully");
    }
}

#[async_trait::async_trait]
impl Routine for TokenPricesRoutine {
    fn name(&self) -> &'static str {
        "TokenPricesRoutine"
    }

    async fn run(&self) -> RoutineResult {
        log::info!("Running TokenPricesRoutine");

        let progress = new_progress(ProgressBar::new_spinner());

        progress.trace("Prices: Creating SpreadsheetManager instance");
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        progress.trace("Prices: 📋 Listing all tokens in the spreadsheet");
        let tokens = self
            .get_token_ids_from_spreadsheet(&spreadsheet_manager)
            .await;

        progress.trace("Prices: ☁️  Getting prices of all tokens from Coingecko");
        let prices = get_token_prices(tokens.as_ref()).await;

        progress.trace("Prices: 📝 Reading the current prices from the spreadsheet");
        let spreadsheet_prices = self
            .get_current_prices_from_spreadsheet(&spreadsheet_manager)
            .await;

        progress.trace("Prices: 📝 Updating the prices on the spreadsheet");
        let new_prices = self.order_prices(&tokens, &prices, spreadsheet_prices);
        self.update_prices_on_spreadsheet(&spreadsheet_manager, new_prices)
            .await;

        progress.info("Prices: ✅ Updated token prices on the spreadsheet");
        finish_progress(&progress);

        Ok(())
    }
}
// async fn run_old(&self) {
//     log::info!("Running UpdateTokenPricesOnSheetsViaCoinGeckoRoutine");

//     // Below: routine to get native token prices from CoinGecko (failed attempt)
//     let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

//     // let coins = CoinGeckoApi.list_coins().await;
//     // let coin_tuples = coins
//     //     .into_iter()
//     //     .filter(|coin| token_ids.contains(&coin.id))
//     //     .map(|coin| (coin.id, coin.symbol.to_uppercase()))
//     //     .collect::<Vec<_>>();
//     let prices = CoinGeckoApi.prices(token_ids.as_ref()).await;

//     let current_prices_on_sheet = spreadsheet_manager
//         .read_named_range(ranges::tokens::RW_PRICES)
//         .await
//         .expect("Should have content")
//         .values
//         .unwrap_or(vec![])
//         .my_into()
//         .into_iter()
//         .map(|x| {
//             x.replace(['$', ','], "")
//                 .parse::<f64>()
//                 .unwrap_or_else(|_| panic!("Should be a number: {}", x))
//         })
//         .collect::<Vec<_>>();

//     let new_prices = token_ids
//         .iter()
//         .enumerate()
//         .map(|(idx, token)| match prices.0.get(token) {
//             Some(price) => price
//                 .usd
//                 .expect("Should have price when PriceResponse exists"),
//             None => current_prices_on_sheet.get(idx).copied().unwrap_or(0.0),
//         })
//         .map(|price| price.to_string())
//         .collect::<Vec<_>>();

//     log::info!("[Coingecko] New prices: {:?}", new_prices);

//     spreadsheet_manager
//         .write_named_range(
//             ranges::tokens::RW_PRICES,
//             ValueRange::from_rows(new_prices.as_ref()),
//         )
//         .await
//         .expect("Should write prices to the spreadsheet");
// }
