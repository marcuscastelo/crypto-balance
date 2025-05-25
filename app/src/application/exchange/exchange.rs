use std::collections::HashMap;

use crate::application::sheets::spreadsheet::BalanceUpdateTarget;

#[async_trait::async_trait]
pub trait ExchangeUseCases: Send + Sync {
    fn exchange_name(&self) -> &'static str;
    fn spreadsheet_target(&self) -> BalanceUpdateTarget;
    async fn fetch_balances(&self) -> anyhow::Result<HashMap<String, f64>>;
}
