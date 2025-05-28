use std::fmt::Display;

// Contains all fields from before, but optional
#[derive(Debug, Clone, Default)]
pub struct TokenInfo {
    pub token_name: Option<String>,
    pub pool: Option<String>,
    pub balance: Option<String>,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub claimable_amount: Option<String>,
    pub end_time: Option<String>,
    pub usd_value: Option<String>,
    pub variant_header: Option<String>, // Supplied, Borrowed, Rewards, etc. (not to be confused with tracking title)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpotTokenInfo {
    pub name: String,
    pub price: String,
    pub amount: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChainProjectInfo {
    pub name: String,
    pub trackings: Vec<ProjectTracking>,
}

#[derive(Debug, Clone)]
pub struct ProjectTracking {
    pub tracking_type: String, // e.g., "Spot", "Lending", "Staked", etc.
    pub token_sections: Vec<ProjectTrackingSection<TokenInfo>>,
}

#[derive(Debug, Clone)]
pub struct ProjectTrackingSection<T> {
    pub title: Option<String>, // e.g., "Supplied", "Borrowed", "Rewards"
    pub tokens: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct LendingTokenInfo {
    pub token_name: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct StakeTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LockedTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub unlock_time: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct RewardTokenInfo {
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct SimpleTokenInfo {
    pub token_name: Option<String>,
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct LiquidityPoolTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct FarmingTokenInfo {
    pub token_name: Option<String>, // When token_name is not available, pool name is used
    pub pool: String,
    pub balance: String,
    pub rewards: Option<String>,
    pub usd_value: String,
}

#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub name: String,
    pub wallet_info: Option<ChainWalletInfo>,
    pub project_info: Vec<ChainProjectInfo>,
}

#[derive(Debug, Clone)]
pub struct ChainWalletInfo {
    pub usd_value: String,
    pub tokens: Vec<SpotTokenInfo>,
}

impl Display for ChainWalletInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tokens = self
            .tokens
            .iter()
            .map(|x| format!("{} {}: {}", x.amount, x.name, x.usd_value))
            .reduce(|a, b| format!("{}; {}", a, b))
            .unwrap_or("<no tokens>".to_string());

        write!(f, "Wallet: {} USD\nTokens: {}", self.usd_value, tokens)
    }
}
