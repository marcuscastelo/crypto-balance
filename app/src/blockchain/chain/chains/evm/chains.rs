use std::sync::LazyLock;

use crate::blockchain::prelude::*;

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
