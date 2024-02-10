use std::{collections::HashMap, sync::LazyLock, vec};

use crate::blockchain::prelude::*;

pub static CELESTIA: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Celestia",
    native_token: Token::Native(NativeTokenName::TIA).into(),
    explorer: &*SCROLLSCAN,
});

pub static COSMOS_CHAINS: LazyLock<HashMap<&'static str, &'static Chain>> = LazyLock::new(|| {
    let chains: Vec<&Chain> = vec![&CELESTIA];

    let mut map = HashMap::new();
    for chain in chains {
        map.insert(chain.name, chain);
    }

    map
});
