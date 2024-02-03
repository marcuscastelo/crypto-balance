use std::sync::Arc;

use crate::blockchain::prelude::*;

#[derive(Debug)]
pub struct Chain {
    pub name: &'static str,
    pub native_token: Arc<Token>,
    pub explorer: &'static dyn BlockExplorer,
}
