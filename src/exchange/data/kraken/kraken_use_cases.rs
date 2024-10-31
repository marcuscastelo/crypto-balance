use crate::{
    exchange::domain::exchange::ExchangeUseCases, sheets::data::spreadsheet::BalanceUpdateTarget,
};

use std::collections::HashMap;

use super::factory::KrakenFactory;

#[allow(unused_imports)]
use num_traits::ToPrimitive;

pub struct KrakenUseCases;

impl KrakenUseCases {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ExchangeUseCases for KrakenUseCases {
    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Kraken
    }

    async fn fetch_balances(&self) -> HashMap<String, f64> {
        let kraken_api = KrakenFactory::create();
        kraken_api
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
            .collect::<HashMap<_, _>>()
    }
}
