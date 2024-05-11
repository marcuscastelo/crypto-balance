use crate::prelude::*;
use crate::price::prelude::CoinGeckoApi;
use google_sheets4::api::ValueRange;
pub struct UpdateTokenPricesOnSheetsViaCoinGeckoRoutine;

#[async_trait::async_trait]
impl Routine for UpdateTokenPricesOnSheetsViaCoinGeckoRoutine {
    async fn run(&self) {
        // Below: routine to get native token prices from CoinGecko (failed attempt)
        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let token_ids: Vec<String> = routines::sheets::SheetsGetTokenIDsRoutine.run().await;

        // let coins = CoinGeckoApi.list_coins().await;
        // let coin_tuples = coins
        //     .into_iter()
        //     .filter(|coin| token_ids.contains(&coin.id))
        //     .map(|coin| (coin.id, coin.symbol.to_uppercase()))
        //     .collect::<Vec<_>>();
        let prices = CoinGeckoApi.prices(token_ids.as_ref()).await;

        let current_prices_on_sheet = spreadsheet_manager
            .read_named_range(ranges::tokens::RW_PRICES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
            .into_iter()
            .map(|x| {
                x.replace(['$', ','], "")
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Should be a number: {}", x))
            })
            .collect::<Vec<_>>();

        let new_prices = token_ids
            .iter()
            .enumerate()
            .map(|(idx, token)| match prices.0.get(token) {
                Some(price) => price
                    .usd
                    .expect("Should have price when PriceResponse exists"),
                None => current_prices_on_sheet.get(idx).copied().unwrap_or(0.0),
            })
            .map(|price| price.to_string())
            .collect::<Vec<_>>();

        spreadsheet_manager
            .write_named_range(
                ranges::tokens::RW_PRICES,
                ValueRange::from_rows(new_prices.as_ref()),
            )
            .await
            .expect("Should write prices to the spreadsheet");
    }
}
