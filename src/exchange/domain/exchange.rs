use std::collections::HashMap;

use crate::sheets::data::spreadsheet::BalanceUpdateTarget;

#[async_trait::async_trait]
pub trait ExchangeUseCases: Send + Sync {
    fn spreadsheet_target(&self) -> BalanceUpdateTarget;
    async fn fetch_balances(&self) -> HashMap<String, f64>;
}
