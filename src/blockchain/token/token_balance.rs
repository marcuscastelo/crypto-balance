use std::sync::Arc;

use crate::prelude::*;

#[derive(Debug)]
pub struct TokenBalance<TokenId = String> {
    pub symbol: TokenId,
    pub balance: f64,
}
