#![allow(clippy::upper_case_acronyms)] // Tokens are upper case acronyms on the crypto space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    ETH,
    WETH,
    MATIC,
}

#[derive(Debug)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: f64, // TODO: Use a more precise type if needed
}
