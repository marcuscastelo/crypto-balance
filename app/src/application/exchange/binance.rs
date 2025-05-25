use std::collections::HashMap;

use crate::infrastructure::exchange::binance_factory::BinanceAccountFactory;
use anyhow::Ok;

use super::use_cases::{BalanceUpdateTarget, ExchangeUseCases};

pub struct BinanceUseCases {
    pub binance_account_factory: BinanceAccountFactory,
}
impl BinanceUseCases {
    pub fn new(binance_account_factory: BinanceAccountFactory) -> Self {
        Self {
            binance_account_factory,
        }
    }
}

#[async_trait::async_trait]
impl ExchangeUseCases for BinanceUseCases {
    fn exchange_name(&self) -> &'static str {
        "Binance"
    }

    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Binance
    }

    async fn fetch_balances(&self) -> anyhow::Result<HashMap<String, f64>> {
        let binance_account = self.binance_account_factory.create();

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
