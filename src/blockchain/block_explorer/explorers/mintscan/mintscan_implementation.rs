use std::{collections::HashMap, sync::Arc};

use crate::BlockExplorer;
use async_trait::async_trait;

use crate::blockchain::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    pub denom: String,
    pub amount: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalancesResponse {
    pub balances: Vec<FetchBalanceResponse>,
    // pub pagination: serde_json::Value, // Ignored for now
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Mintscan;

const MINTSCAN_BANK_BASE_URL: &str = "https://lcd-celestia.cosmostation.io/cosmos/bank/v1beta1";

#[async_trait]
impl BlockExplorer for Mintscan {
    async fn fetch_native_balance(&self, cosmos_address: &str) -> TokenBalance {
        let url = format!("{MINTSCAN_BANK_BASE_URL}/balances/{cosmos_address}",);
        let resp = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::ORIGIN, "https://www.mintscan.io")
            .header(reqwest::header::REFERER, "https://www.mintscan.io")
            .send()
            .await
            .expect("Should send request and receive response")
            .text() // TODO: change to json to avoid double parsing (also on other block explorers)
            .await
            .expect("Should receive response as text");

        let resp: FetchBalancesResponse =
            serde_json::from_str(&resp).expect("Should parse response as JSON");
        let balance = resp
            .balances
            .iter()
            .map(|b| b.amount.parse::<f64>().unwrap())
            .reduce(|a, b| a + b)
            .unwrap_or(0f64)
            / (WEI_CONVERSION as f64);

        TokenBalance {
            token: self.get_chain().native_token.to_owned(),
            balance,
        }
    }

    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> TokenBalance {
        todo!()
    }

    async fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        todo!()
    }

    fn get_chain(&self) -> &'static Chain {
        &CELESTIA
    }
}
