use std::sync::Arc;

use crate::prelude::*;

#[derive(Debug)]
pub struct Network {
    pub name: &'static str,
    pub native_token: Arc<Token>,
    pub explorer: &'static dyn BlockExplorer,
}
