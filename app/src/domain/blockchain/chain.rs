use std::sync::Arc;

use super::{explorer::BlockExplorer, token::Token};

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: &'static str,
    pub native_token: Arc<Token>,
    pub explorer: &'static dyn BlockExplorer,
}
