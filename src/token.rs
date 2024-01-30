#![allow(clippy::upper_case_acronyms)]

use std::sync::Arc; // Tokens are upper case acronyms on the crypto space

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC20TokenInfo {
    pub token_name: Box<str>,
    pub token_symbol: Box<str>,
    pub contract_address: Box<str>,
    pub token_decimal: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NativeTokenName {
    ETH,
    MATIC,
    BNB,
    BTC,
    SOL,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Native(NativeTokenName),
    ERC20(ERC20TokenInfo),
}

#[derive(Debug)]
pub struct TokenBalance {
    pub token: Arc<Token>,
    pub balance: f64, // TODO: Use a more precise type if needed
}
