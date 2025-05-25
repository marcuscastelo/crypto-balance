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

    pub mod hold {
        pub const RW_DATA: &str = "Balance_Hold__mData";
    }
}

#[allow(non_snake_case)] // To match sheet names
pub mod AaH {
    // USDT
    pub const RW_USDT_BALANCES_NAMES: &str = "AaH__vUsdtBalances_Names";

    // Ethereum
    pub const RW_ETH_BALANCES_NAMES: &str = "AaH__vEthBalances_Names";

    // Pendle
    pub const RW_PENDLE_BALANCES_NAMES: &str = "AaH__vPendleBalances_Names";

    // Bitcoin
    pub const RW_BTC_BALANCES_NAMES: &str = "AaH__vBtcBalances";

    // Ethena
    pub const RW_ENA_BALANCES_NAMES: &str = "AaH__vEnaBalances_Names";

    // ETHFI
    pub const RW_ETHFI_BALANCES_NAMES: &str = "AaH__vEthfiBalances_Names";

    // GammaSwap (GS)
    pub const RW_GS_BALANCES_NAMES: &str = "AaH__vGsBalances_Names";

    // Tango
    pub const RW_TANGO_BALANCES_NAMES: &str = "AaH__vTangoBalances_Names";

    // Pear
    pub const RW_PEAR_BALANCES_NAMES: &str = "AaH__vPearBalances_Names";

    // Instadapp
    pub const RW_INST_BALANCES_NAMES: &str = "AaH__vInstBalances_Names";

    // Spectra
    pub const RW_SPECTRA_BALANCES_NAMES: &str = "AaH__vSpectraBalances_Names";

    // Hyperliquid
    pub const RW_HYPE_BALANCES_NAMES: &str = "AaH__vHypeBalances_Names";
}

pub mod airdrops {
    pub const RW_DEBANK_TOTAL_USD: &str = "Airdrops__cDebankTotalUSD";
}
