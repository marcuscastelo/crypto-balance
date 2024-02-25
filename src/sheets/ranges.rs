// For now, we are going to hardcode the ranges from sheets to minimize code complexity

pub mod tokens {
    pub const RO_IDS: &str = "Tokens__vIDs";
    pub const RO_NAMES: &str = "Tokens__vNames";
    pub const RW_PRICES: &str = "Tokens__vPrices";
}

pub mod balances {
    pub mod binance {
        pub const RW_AMOUNTS: &str = "Balance_Binance__vAmounts";
    }

    pub mod kraken {
        pub const RW_AMOUNTS: &str = "Balance_Kraken__vAmounts";
    }
}
