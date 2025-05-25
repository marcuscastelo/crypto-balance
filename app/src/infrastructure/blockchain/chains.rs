use std::sync::LazyLock;

use crate::domain::blockchain::{
    chain::Chain,
    token::{NativeTokenSymbol, Token},
};

use super::explorers::etherscan::{
    arbiscan::ARBISCAN, optimistic_etherscan::OPTIMISTIC_ETHERSCAN, polygonscan::POLYGONSCAN,
};

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
