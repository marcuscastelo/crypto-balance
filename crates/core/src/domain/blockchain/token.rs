// Tokens are upper case acronyms on the crypto space
#![allow(clippy::upper_case_acronyms)]

use strum::EnumString;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC20TokenInfo {
    pub token_name: Box<str>,
    pub token_symbol: Box<str>,
    pub contract_address: Box<str>,
    pub token_decimal: Box<str>,
}

#[derive(strum::Display, Debug, Clone, PartialEq, Eq, Hash, EnumString)]
pub enum NativeTokenSymbol {
    ETH,
    MATIC,
    BNB,
    BTC,
    SOL,
    ATOM,
    OSMO,
    TIA,
    KUJI,
    INJ,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Native(NativeTokenSymbol),
    ERC20(ERC20TokenInfo),
}

impl Token {
    pub fn symbol(&self) -> String {
        match self {
            Token::Native(symbol) => symbol.to_string(),
            Token::ERC20(info) => info.token_symbol.to_string(),
        }
    }
}
