use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::blockchain::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchNativeBalanceResponse {
    coin_balance: String,
}

#[derive(Debug)]
pub struct ZoraExplorer;

#[async_trait]
impl BlockExplorer for ZoraExplorer {
    async fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!("https://explorer.zora.energy/api/v2/addresses/{evm_address}",);
        let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
        let resp: FetchNativeBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.coin_balance.parse::<f64>().unwrap() / (WEI_CONVERSION as f64);

        TokenBalance {
            token: self.get_chain().native_token.to_owned(),
            balance,
        }
    }

    async fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        todo!()
    }

    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> TokenBalance {
        todo!()
    }

    fn get_chain(&self) -> &'static Chain {
        &ZORA
    }
}
