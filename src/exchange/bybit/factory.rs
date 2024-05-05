use std::sync::Arc;

use crate::prelude::*;

use ::bybit_rs::bybit::{account::Account, http_manager::HttpManager};
use bybit_rs::bybit::account::AccountHTTP;

pub struct BybitFactory;

impl BybitFactory {
    pub fn create() -> AccountHTTP {
        let http_manager = Arc::new(HttpManager::new(
            CONFIG.bybit.api_key.to_string(),
            CONFIG.bybit.secret_key.to_string(),
            false,
        ));

        AccountHTTP::new(http_manager)
    }
}
