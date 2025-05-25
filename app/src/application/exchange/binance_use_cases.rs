use std::collections::HashMap;

use error_stack::{Report, ResultExt};

use crate::{
    domain::exchange::BalanceUpdateTarget,
    infrastructure::exchange::binance_factory::BinanceAccountFactory,
};

use super::use_cases::{ExchangeUseCases, ExchangeUseCasesError};

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

    async fn fetch_balances(
        &self,
    ) -> error_stack::Result<HashMap<String, f64>, ExchangeUseCasesError> {
        let binance_account = self.binance_account_factory.create();

        let balances = binance_account
            .get_account()
            .await
            .map_err(Report::from)
            .change_context(ExchangeUseCasesError::FetchBalancesError("Binance"))?
            .balances
            .into_iter()
            .filter(|x| x.free > 0.0)
            // Convert to Hashmap of token.asset, token.free
            .map(|token| (token.asset, token.free))
            .collect::<HashMap<_, _>>();

        Ok(balances)
    }
}
