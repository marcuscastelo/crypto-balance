#![allow(clippy::upper_case_acronyms)] // Tokens are upper case acronyms on the crypto space
pub enum TokenType {
    Native,
    ERC20,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC20TokenInfo {
    pub token_name: String,
    pub token_symbol: String,
    pub contract_address: String,
    pub token_decimal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NativeTokenName {
    ETH,
    MATIC,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Native(NativeTokenName),
    ERC20(ERC20TokenInfo),
}

#[derive(Debug)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: f64, // TODO: Use a more precise type if needed
}
