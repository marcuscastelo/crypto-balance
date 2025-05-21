use crate::{
    exchange::domain::exchange::ExchangeUseCases, sheets::data::spreadsheet::BalanceUpdateTarget,
};

use std::collections::HashMap;

use super::factory::KrakenFactory;

use anyhow::Ok;
#[allow(unused_imports)]
use num_traits::ToPrimitive;

pub struct KrakenUseCases;

#[async_trait::async_trait]
impl ExchangeUseCases for KrakenUseCases {
    fn exchange_name(&self) -> &'static str {
        "Kraken"
    }

    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Kraken
    }

    async fn fetch_balances(&self) -> anyhow::Result<HashMap<String, f64>> {
        let kraken_api = KrakenFactory::create();
        let balances = kraken_api
            .get_account_balance()
            .await
            .unwrap()
            .into_iter()
            .map(|(symbol, amount)| {
                (
                    match symbol.as_str() {
                        "XXBT" => "BTC".to_string(),
                        "XETH" => "ETH".to_string(),
                        "XXRP" => "XRP".to_string(),
                        "ZUSD" => "USDT".to_string(),
                        _ => symbol,
                    },
                    amount.to_f64().expect("Should be convertible to f64"),
                )
            })
            .filter(|(_, amount)| *amount > 0.0)
            .collect::<HashMap<_, _>>();

        Ok(balances)
    }
}
