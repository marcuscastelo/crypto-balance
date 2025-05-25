use std::collections::HashMap;

use crate::domain::sheets::ranges;

#[async_trait::async_trait]
pub trait ExchangeUseCases: Send + Sync {
    fn exchange_name(&self) -> &'static str;
    fn spreadsheet_target(&self) -> BalanceUpdateTarget;
    async fn fetch_balances(&self) -> anyhow::Result<HashMap<String, f64>>;
}

pub enum BalanceUpdateTarget {
    Binance,
    Kraken,
}

pub fn get_target_range(target: BalanceUpdateTarget) -> &'static str {
    match target {
        BalanceUpdateTarget::Binance => ranges::balances::binance::RW_AMOUNTS,
        BalanceUpdateTarget::Kraken => ranges::balances::kraken::RW_AMOUNTS,
    }
}
