#[derive(Debug, Clone)]
pub struct TokenBalance<TokenId = String> {
    pub symbol: TokenId,
    pub balance: f64,
}
