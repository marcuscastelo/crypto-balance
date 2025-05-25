use std::collections::HashMap;

use crate::{
    application::sheets::spreadsheet::BalanceUpdateTarget,
    infrastructure::exchange::binance_factory::BinanceAccountFactory,
};
use anyhow::Ok;

use super::exchange::ExchangeUseCases;

pub struct BinanceUseCases;

#[async_trait::async_trait]
impl ExchangeUseCases for BinanceUseCases {
    fn exchange_name(&self) -> &'static str {
        "Binance"
    }

    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Binance
    }

    async fn fetch_balances(&self) -> anyhow::Result<HashMap<String, f64>> {
        let binance_account = BinanceAccountFactory::create();

        let balances = binance_account
            .get_account()
            .await
            .unwrap()
            .balances
            .into_iter()
            .filter(|x| x.free > 0.0)
            // Convert to Hashmap of token.asset, token.free
            .map(|token| (token.asset, token.free))
            .collect::<HashMap<_, _>>();

        Ok(balances)
    }
}
