use std::collections::HashMap;

#[allow(unused_imports)]
use num_traits::ToPrimitive;

use crate::{
    domain::exchange::BalanceUpdateTarget, infrastructure::exchange::kraken_factory::KrakenFactory,
};
use error_stack::{report, ResultExt};

use super::use_cases::{ExchangeUseCases, ExchangeUseCasesError};

pub struct KrakenUseCases {
    pub kraken_factory: KrakenFactory,
}

impl KrakenUseCases {
    pub fn new(kraken_factory: KrakenFactory) -> Self {
        Self { kraken_factory }
    }
}

#[async_trait::async_trait]
impl ExchangeUseCases for KrakenUseCases {
    fn exchange_name(&self) -> &'static str {
        "Kraken"
    }

    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Kraken
    }

    async fn fetch_balances(
        &self,
    ) -> error_stack::Result<HashMap<String, f64>, ExchangeUseCasesError> {
        let kraken_api = self.kraken_factory.create();
        let balances = kraken_api
            .get_account_balance()
            .await
            .map_err(|error| report!(ExchangeUseCasesError::InternalError(format!("{error:?}"))))
            .change_context(ExchangeUseCasesError::FetchBalancesError(
                "Failed to fetch balances from Kraken",
            ))?
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
