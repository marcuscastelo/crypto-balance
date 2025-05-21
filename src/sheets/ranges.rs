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

    pub mod bybit {
        pub const RW_AMOUNTS: &str = "Balance_Bybit__vAmounts";
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
    pub const RW_USDT_BALANCES_AMOUNTS: &str = "AaH__vUsdtBalances_Amounts";

    // Ethereum
    pub const RW_ETH_BALANCES_NAMES: &str = "AaH__vEthBalances_Names";
    pub const RW_ETH_BALANCES_AMOUNTS: &str = "AaH__vEthBalances_Amounts";

    // Pendle
    pub const RW_PENDLE_BALANCES_NAMES: &str = "AaH__vPendleBalances_Names";
    pub const RW_PENDLE_BALANCES_AMOUNTS: &str = "AaH__vPendleBalances_Amounts";

    // Bitcoin
    pub const RW_BTC_BALANCES_NAMES: &str = "AaH__vBtcBalances_Names";
    pub const RW_BTC_BALANCES_AMOUNTS: &str = "AaH__vBtcBalances_Amounts";

    // Ethena
    pub const RW_ENA_BALANCES_NAMES: &str = "AaH__vEnaBalances_Names";
    pub const RW_ENA_BALANCES_AMOUNTS: &str = "AaH__vEnaBalances_Amounts";

    // ETHFI
    pub const RW_ETHFI_BALANCES_NAMES: &str = "AaH__vEthfiBalances_Names";
    pub const RW_ETHFI_BALANCES_AMOUNTS: &str = "AaH__vEthfiBalances_Amounts";

    // GammaSwap (GS)
    pub const RW_GS_BALANCES_NAMES: &str = "AaH__vGsBalances_Names";
    pub const RW_GS_BALANCES_AMOUNTS: &str = "AaH__vGsBalances_Amounts";

    // Tango
    pub const RW_TANGO_BALANCES_NAMES: &str = "AaH__vTangoBalances_Names";
    pub const RW_TANGO_BALANCES_AMOUNTS: &str = "AaH__vTangoBalances_Amounts";

    // Pear
    pub const RW_PEAR_BALANCES_NAMES: &str = "AaH__vPearBalances_Names";
    pub const RW_PEAR_BALANCES_AMOUNTS: &str = "AaH__vPearBalances_Amounts";

    // Instadapp
    pub const RW_INST_BALANCES_NAMES: &str = "AaH__vInstBalances_Names";
    pub const RW_INST_BALANCES_AMOUNTS: &str = "AaH__vInstBalances_Amounts";

    // Spectra
    pub const RW_SPECTRA_BALANCES_NAMES: &str = "AaH__vSpectraBalances_Names";
    pub const RW_SPECTRA_BALANCES_AMOUNTS: &str = "AaH__vSpectraBalances_Amounts";

    // Hyperliquid
    pub const RW_HYPE_BALANCES_NAMES: &str = "AaH__vHypeBalances_Names";
    pub const RW_HYPE_BALANCES_AMOUNTS: &str = "AaH__vHypeBalances_Amounts";
}

pub mod airdrops {
    pub const RW_DEBANK_TOTAL_USD: &str = "Airdrops__cDebankTotalUSD";
    pub const RW_SONAR_WATCH_TOTAL_USD: &str = "Airdrops__cSonarWatchTotalUSD";
}
