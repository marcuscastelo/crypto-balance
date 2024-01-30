use std::{collections::HashMap, sync::LazyLock, vec};

use crate::prelude::*;
use lazy_static::lazy_static;

pub static ETHEREUM: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Ethereum",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*ETHERSCAN,
});

pub static ARBITRUM: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Arbitrum",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*ARBISCAN,
});

pub static OPTIMISM: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Optimism",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*OPTIMISTIC_ETHERSCAN,
});

pub static POLYGON: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Polygon",
    native_token: Token::Native(NativeTokenName::MATIC).into(),
    explorer: &*POLYGONSCAN,
});

pub static BASE: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Base",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*BASESCAN,
});

pub static LINEA: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Linea",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*LINEASCAN,
});

pub static SOLANA: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Solana",
    native_token: Token::Native(NativeTokenName::SOL).into(),
    explorer: todo!("Solana explorer"),
});

pub static ZKSYNC: LazyLock<Network> = LazyLock::new(|| Network {
    name: "zkSync",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &ZkSyncExplorer,
});

pub static ZORA: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Zora",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &ZoraExplorer,
});

pub static SCROLL: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Scroll",
    native_token: Token::Native(NativeTokenName::ETH).into(),
    explorer: &*SCROLLSCAN,
});

pub static BINANCE: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Binance Smart Chain",
    native_token: Token::Native(NativeTokenName::BNB).into(),
    explorer: todo!("Binance explorer"),
});

pub static BITCOIN: LazyLock<Network> = LazyLock::new(|| Network {
    name: "Bitcoin",
    native_token: Token::Native(NativeTokenName::BTC).into(),
    explorer: todo!("Bitcoin explorer"),
});

pub static NETWORKS: LazyLock<HashMap<&'static str, &'static Network>> = LazyLock::new(|| {
    let networks: Vec<&Network> = vec![
        &ETHEREUM, //
        &ARBITRUM, //
        &OPTIMISM, //
        &POLYGON,  //
        &BASE,     //
        &LINEA,    //
        // &SOLANA, //
        // &ZKSYNC, //
        // &ZORA,   //
        &SCROLL, //
                 // &BINANCE, //
                 // &BITCOIN, //
    ];

    let mut map = HashMap::new();
    for network in networks {
        map.insert(network.name, network);
    }

    map
});
