use std::sync::Arc;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct TokenBalance<TokenId = String> {
    pub symbol: TokenId,
    pub balance: f64,
}
