#[derive(serde::Deserialize, Debug, Clone)]
pub struct BlockchainConfig {
    pub etherscan_api_key: Box<str>,
    pub scrollscan_api_key: Box<str>,
    pub lineascan_api_key: Box<str>,
    pub basescan_api_key: Box<str>,
    pub arbiscan_api_key: Box<str>,
    pub optimistic_etherscan_api_key: Box<str>,
    pub polygonscan_api_key: Box<str>,
    pub hold: HoldBlockchainConfig,
    pub hold_sc: HoldBlockchainConfig,
    pub airdrops: AirdropsBlockchainConfig,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct HoldBlockchainConfig {
    pub evm: EvmBlockchainConfig,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AirdropsBlockchainConfig {
    pub evm: EvmBlockchainConfig,
    pub solana: SolanaBlockchainConfig,
    pub cosmos: CosmosBlockchainConfig,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct EvmBlockchainConfig {
    pub address: Box<str>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SolanaBlockchainConfig {
    pub address: Box<str>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CosmosBlockchainConfig {
    pub cosmos_address: Box<str>,
    pub osmosis_address: Box<str>,
    pub celestia_address: Box<str>,
    pub injective_address: Box<str>,
}
