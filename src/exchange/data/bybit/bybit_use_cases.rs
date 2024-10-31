use std::collections::HashMap;

use bybit_rs::bybit::account::Account;

use crate::{
    exchange::domain::exchange::ExchangeUseCases, sheets::data::spreadsheet::BalanceUpdateTarget,
};

use super::factory::BybitFactory;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceCoin {
    coin: String,
    equity: String,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceAccount {
    coin: Vec<BybitGetWalletBalanceCoin>,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceResult {
    list: Vec<BybitGetWalletBalanceAccount>,
}

#[derive(Debug, Deserialize)]
struct BybitGetWalletBalanceResponse {
    result: BybitGetWalletBalanceResult,
}

pub struct BybitUseCases;

#[async_trait::async_trait]
impl ExchangeUseCases for BybitUseCases {
    fn spreadsheet_target(&self) -> BalanceUpdateTarget {
        BalanceUpdateTarget::Bybit
    }

    async fn fetch_balances(&self) -> HashMap<String, f64> {
        let bybit_account = BybitFactory::create();
        let response_value = bybit_account
            .get_wallet_balance(HashMap::from([(
                "accountType".to_owned(),
                "UNIFIED".to_owned(),
            )]))
            .await
            .expect("Should get wallet balance");

        let balances: BybitGetWalletBalanceResponse =
            serde_json::from_value(response_value).expect("Should deserialize response");

        let balances = HashMap::from(
            balances
                .result
                .list
                .iter()
                .map(|account| {
                    // All account coins as (coin, equity)
                    account
                        .coin
                        .iter()
                        .map(|coin| (coin.coin.clone(), coin.equity.parse::<f64>().unwrap()))
                })
                .flatten()
                .collect::<HashMap<_, _>>(),
        );
        balances
    }
}
