use std::collections::HashMap;

#[async_trait::async_trait]
pub trait ExchangeUseCases: Send + Sync {
    async fn get_balances(&self) -> HashMap<String, f64>;
}
