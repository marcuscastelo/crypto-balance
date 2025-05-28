#[derive(Debug, Clone)]
/// Represents a chain in the DeBank ecosystem (e.g., Ethereum, Binance Smart Chain, etc.)
/// Each chain is composed of one wallet (the user's tokens) and multiple projects (e.g., Aave, Compound, etc.).
pub struct Chain {
    pub name: String,

    /// Chain wallet tokens
    pub wallet_info: Option<ChainWallet>,
    pub project_info: Vec<Project>,
}

#[derive(Debug, Clone)]
pub struct ChainWallet {
    pub usd_value: String,
    pub tokens: Vec<SpotTokenInfo>,
}

#[derive(Debug, Clone)]
/// Represents a project in the DeBank ecosystem (e.g., Aave, Compound, etc.)
/// A project is limited to a single chain, so if a project exists on multiple chains, they will be represented as separate `Project` instances.
pub struct Project {
    pub name: String,

    /// One single project can have multiple features, like lending, staking, farming, etc. Each feature is represented as a `ProjectTracking`.
    pub trackings: Vec<ProjectTracking>,
}

#[derive(Debug, Clone)]
/// Represents a specific type of tracking for a project, such as lending, staking, farming, etc.
pub struct ProjectTracking {
    /// Type of tracking. Possible values include "Lending", "Staking", "Farming", "Liquidity Pool", etc.
    /// TODO: Use an enum for better type safety.
    pub tracking_type: String,

    /// Sections for this tracking type.
    /// It is only needed for now for "Lending" type, which has "Supplied", "Borrowed", and "Rewards" sections.
    pub token_sections: Vec<ProjectTrackingSection>,
}

#[derive(Debug, Clone)]
/// Represents a section within a project tracking, such as "Supplied", "Borrowed", "Rewards", etc.
/// Most trackings will have only one section, but some (like "Lending") can have multiple sections.
pub struct ProjectTrackingSection {
    pub title: String,

    /// Tokens in this section.
    pub tokens: Vec<TokenInfo>,
}

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
pub struct SpotTokenInfo {
    pub name: String,
    pub price: String,
    pub amount: String,
    pub usd_value: String,
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
pub struct SimpleTokenInfo {
    pub token_name: Option<String>,
    pub pool: String,
    pub balance: String,
    pub usd_value: String,
}
