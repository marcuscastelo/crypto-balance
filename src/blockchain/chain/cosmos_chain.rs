use std::sync::Arc;

use crate::blockchain::prelude::*;

// {
//     "$schema": "../chain.schema.json",
//     "chain_name": "osmosis",
//     "status": "live",
//     "website": "https://osmosis.zone/",
//     "network_type": "mainnet",
//     "pretty_name": "Osmosis",
//     "chain_id": "osmosis-1",
//     "bech32_prefix": "osmo",
//     "daemon_name": "osmosisd",
//     "node_home": "$HOME/.osmosisd",
//     "key_algos": [
//       "secp256k1"
//     ],
//     "slip44": 118,
//     "fees": {
//       "fee_tokens": [
//         {
//           "denom": "uosmo",
//           "fixed_min_gas_price": 0,
//           "low_gas_price": 0,
//           "average_gas_price": 0.025,
//           "high_gas_price": 0.04
//         }
//       ]
//     },
//     "staking": {
//       "staking_tokens": [
//         {
//           "denom": "uosmo"
//         }
//       ],
//       "lock_duration": {
//         "time": "1209600s"
//       }
//     },
//     "codebase": {
//       "git_repo": "https://github.com/osmosis-labs/osmosis",
//       "recommended_version": "v12.2.0",
//       "compatible_versions": [
//         "v12.1.0"
//         "v12.2.0"
//       ],
//       "cosmos_sdk_version": "0.46",
//       "consensus": {
//         "type": "tendermint",
//         "version": "0.34"
//       },
//       "cosmwasm_version": "0.28",
//       "cosmwasm_enabled": true,
//       "ibc_go_version": "3.0.0",
//       "ics_enabled": [
//         "ics20-1"
//       ],
//       "genesis": {
//         "name": "v3",
//         "genesis_url": "https://github.com/osmosis-labs/networks/raw/main/osmosis-1/genesis.json"
//       },
//       "versions": [
//         {
//           "name": "v3",
//           "tag": "v3.1.0",
//           "height": 0,
//           "next_version_name": "v4"
//         },
//         {
//           "name": "v4",
//           "tag": "v4.2.0",
//           "height": 1314500,
//           "proposal": 38,
//           "next_version_name": "v5"
//         },
//         {
//           "name": "v5",
//           "tag": "v6.4.1",
//           "height": 2383300,
//           "proposal": 95,
//           "next_version_name": "v7"
//         },
//         {
//           "name": "v7",
//           "tag": "v8.0.0",
//           "height": 3401000,
//           "proposal": 157,
//           "next_version_name": "v9"
//         },
//         {
//           "name": "v9",
//           "tag": "v10.0.1",
//           "height": 4707300,
//           "proposal": 252,
//           "next_version_name": "v11"
//         },
//         {
//           "name": "v11",
//           "tag": "v11.0.0",
//           "height": 5432450,
//           "proposal": 296,
//           "next_version_name": "v12"
//         },
//         {
//           "name": "v12",
//           "tag": "v12.1.0",
//           "height": 6246000,
//           "proposal": 335,
//           "recommended_version": "v12.2.0",
//           "compatible_versions": [
//             "v12.1.0"
//             "v12.2.0"
//           ],
//           "cosmos_sdk_version": "0.46",
//           "consensus": {
//             "type": "tendermint",
//             "version": "0.34"
//           },
//           "cosmwasm_version": "0.28",
//           "cosmwasm_enabled": true,
//           "ibc_go_version": "3.0.0",
//           "ics_enabled": [
//             "ics20-1"
//           ],
//           "next_version_name": "v13"
//         }
//       ]
//     },
//     "images": [
//       {
//         "png": "https://raw.githubusercontent.com/cosmos/chain-registry/master/osmosis/images/osmosis-chain-logo.png",
//         "theme": {
//           "primary_color_hex": "#231D4B"
//         }
//       }
//     ],
//     "peers": {
//       "seeds": [
//         {
//           "id": "83adaa38d1c15450056050fd4c9763fcc7e02e2c",
//           "address": "ec2-44-234-84-104.us-west-2.compute.amazonaws.com:26656",
//           "provider": "notional"
//         },
//         {
//           "id": "f515a8599b40f0e84dfad935ba414674ab11a668",
//           "address": "osmosis.blockpane.com:26656",
//           "provider": "blockpane"
//         }
//       ],
//       "persistent_peers": [
//         {
//           "id": "8f67a2fcdd7ade970b1983bf1697111d35dfdd6f",
//           "address": "52.79.199.137:26656",
//           "provider": "cosmostation"
//         },
//         {
//           "id": "8d9967d5f865c68f6fe2630c0f725b0363554e77",
//           "address": "134.255.252.173:26656",
//           "provider": "divecrypto"
//         },
//         ...
//         ...
//         {
//           "id": "64d36f3a186a113c02db0cf7c588c7c85d946b5b",
//           "address": "209.97.132.170:26656",
//           "provider": "solidstake"
//         },
//         {
//           "id": "4d9ac3510d9f5cfc975a28eb2a7b8da866f7bc47",
//           "address": "37.187.38.191:26656",
//           "provider": "stakelab"
//         }
//       ]
//     },
//     "apis": {
//       "rpc": [
//         {
//           "address": "https://osmosis.validator.network/",
//           "provider": "validatornetwork"
//         },
//         {
//           "address": "https://rpc-osmosis.blockapsis.com",
//           "provider": "chainapsis"
//         }
//       ],
//       "rest": [
//         {
//           "address": "https://lcd-osmosis.blockapsis.com",
//           "provider": "chainapsis"
//         }
//       ],
//       "grpc": [
//         {
//           "address": "osmosis.strange.love:9090",
//           "provider": "strangelove"
//         }
//       ]
//     },
//     "explorers": [
//       {
//         "kind": "mintscan",
//         "url": "https://www.mintscan.io/osmosis",
//         "tx_page": "https://www.mintscan.io/osmosis/txs/${txHash}",
//         "account_page": "https://www.mintscan.io/osmosis/account/${accountAddress}"
//       }
//     ],
//     "keywords": [
//       "dex"
//     ]
//   }

pub struct CosmosChain {
    name: &'static str,
    native_token: Arc<Token>,
    explorer: &'static dyn BlockExplorer,
}
