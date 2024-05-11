use std::{collections::HashMap, sync::LazyLock, vec};

use crate::blockchain::prelude::*;

use self::block_explorer::explorers::mintscan::mintscan_implementation::Mintscan;

pub static COSMOS_HUB_MINTSCAN_EXPLORER: Mintscan = Mintscan {
    lcd_url: "https://lcd-cosmos.cosmostation.io",
    chain: LazyLock::new(|| &COSMOS_HUB),
};

pub static COSMOS_HUB: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Cosmos Hub",
    native_token: Token::Native(NativeTokenSymbol::ATOM).into(),
    explorer: &COSMOS_HUB_MINTSCAN_EXPLORER,
});

pub static OSMOSIS_MINTSCAN_EXPLORER: Mintscan = Mintscan {
    lcd_url: "https://lcd-osmosis.cosmostation.io",
    chain: LazyLock::new(|| &OSMOSIS),
};

pub static OSMOSIS: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Osmosis",
    native_token: Token::Native(NativeTokenSymbol::OSMO).into(),
    explorer: &OSMOSIS_MINTSCAN_EXPLORER,
});

pub static CELESTIA_MINTSCAN_EXPLORER: Mintscan = Mintscan {
    lcd_url: "https://lcd-celestia.cosmostation.io",
    chain: LazyLock::new(|| &CELESTIA),
};

pub static CELESTIA: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Celestia",
    native_token: Token::Native(NativeTokenSymbol::TIA).into(),
    explorer: &CELESTIA_MINTSCAN_EXPLORER,
});

pub static INJECTIVE_MINTSCAN_EXPLORER: Mintscan = Mintscan {
    lcd_url: "https://lcd-injective.cosmostation.io",
    chain: LazyLock::new(|| &INJECTIVE),
};

pub static INJECTIVE: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Injective",
    native_token: Token::Native(NativeTokenSymbol::INJ).into(),
    explorer: &INJECTIVE_MINTSCAN_EXPLORER,
});

pub static COSMOS_CHAINS: LazyLock<HashMap<&'static str, &'static Chain>> = LazyLock::new(|| {
    let chains: Vec<&Chain> = vec![&COSMOS_HUB, &OSMOSIS, &CELESTIA, &INJECTIVE];

    let mut map = HashMap::new();
    for chain in chains {
        map.insert(chain.name, chain);
    }

    map
});
