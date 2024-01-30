use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::WEI_CONVERSION;
use crate::network::networks::ZORA;
use crate::token::{ERC20TokenInfo, Token, TokenBalance};

use crate::block_explorer::explorer::BlockExplorer;
use crate::Network;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchNativeBalanceResponse {
    coin_balance: String,
}

#[derive(Debug)]
pub struct ZoraExplorer;

impl BlockExplorer for ZoraExplorer {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!("https://explorer.zora.energy/api/v2/addresses/{evm_address}",);
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchNativeBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.coin_balance.parse::<f64>().unwrap() / (WEI_CONVERSION as f64);

        TokenBalance {
            token: self.get_network().native_token.to_owned(),
            balance,
        }
    }

    fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        todo!()
    }

    fn fetch_erc20_balance(&self, evm_address: &str, token_info: ERC20TokenInfo) -> TokenBalance {
        todo!()
    }

    fn get_network(&self) -> &'static Network {
        &ZORA
    }
}
