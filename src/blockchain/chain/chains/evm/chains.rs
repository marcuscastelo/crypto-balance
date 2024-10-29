use std::{collections::HashMap, sync::LazyLock, vec};

use crate::blockchain::prelude::*;

pub static ETHEREUM: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Ethereum",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*ETHERSCAN,
});

pub static ARBITRUM: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Arbitrum",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*ARBISCAN,
});

pub static OPTIMISM: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Optimism",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*OPTIMISTIC_ETHERSCAN,
});

pub static POLYGON: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Polygon",
    native_token: Token::Native(NativeTokenSymbol::MATIC).into(),
    explorer: &*POLYGONSCAN,
});

pub static BASE: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Base",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*BASESCAN,
});

pub static LINEA: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Linea",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*LINEASCAN,
});

pub static SOLANA: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Solana",
    native_token: Token::Native(NativeTokenSymbol::SOL).into(),
    explorer: todo!("Solana explorer"),
});

pub static ZKSYNC: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "zkSync",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &ZkSyncExplorer,
});

pub static ZORA: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Zora",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &ZoraExplorer,
});

pub static SCROLL: LazyLock<Chain> = LazyLock::new(|| Chain {
    name: "Scroll",
    native_token: Token::Native(NativeTokenSymbol::ETH).into(),
    explorer: &*SCROLLSCAN,
});

pub static EVM_CHAINS: LazyLock<HashMap<&'static str, &'static Chain>> = LazyLock::new(|| {
    let chains: Vec<&Chain> = vec![
        &ETHEREUM, //
        &ARBITRUM, //
        &OPTIMISM, //
        &POLYGON,  //
        &BASE,     //
        &LINEA,    //
        // &ZKSYNC, //
        // &ZORA,   //
        &SCROLL, //
    ];

    let mut map = HashMap::new();
    for chain in chains {
        map.insert(chain.name, chain);
    }

    map
});
