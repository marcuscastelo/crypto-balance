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

pub mod spam_filter {
    pub fn check_spam(token_symbol: &str, token_name: &str) -> bool {
        let has_visit = |s: &str| s.to_uppercase().contains("VISIT");
        let has_access =
            |s: &str| s.to_uppercase().contains("ACCES") || s.to_uppercase().contains("ACESS");

        let has_www = |s: &str| s.to_uppercase().contains("WWW");
        let has_com = |s: &str| s.to_uppercase().contains(".COM");
        let has_io = |s: &str| s.to_uppercase().contains(".IO");
        let has_eligible = |s: &str| s.to_uppercase().contains("ELIGIBLE");
        let has_airdrop = |s: &str| s.to_uppercase().contains("AIRDROP");
        let has_claim = |s: &str| s.to_uppercase().contains("CLAIM");
        let has_free = |s: &str| s.to_uppercase().contains("FREE");
        let has_voucher = |s: &str| s.to_uppercase().contains("VOUCHER");

        let has_spam = |s: &str| {
            has_visit(s)
                || has_access(s)
                || has_www(s)
                || has_com(s)
                || has_io(s)
                || has_eligible(s)
                || has_airdrop(s)
                || has_claim(s)
                || has_free(s)
                || has_voucher(s)
        };

        has_spam(token_symbol) || has_spam(token_name)
    }
}
