use crate::blockchain::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchNativeBalanceResponse {
    status: String,
    message: String,
    result: String,
}

#[derive(Debug)]
pub struct ZkSyncExplorer;

#[async_trait]
impl BlockExplorer for ZkSyncExplorer {
    async fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!(
            "https://block-explorer-api.mainnet.zksync.io/api\
                ?module=account\
                &action=balance\
                &address={evm_address}"
        );
        let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
        let resp: FetchNativeBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.result.parse::<f64>().unwrap() / WEI_CONVERSION;

        TokenBalance {
            token: self.get_chain().native_token.to_owned(),
            balance,
        }
    }

    async fn fetch_erc20_balances(&self, _evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        todo!()
    }

    async fn fetch_erc20_balance(
        &self,
        _evm_address: &str,
        _token_info: ERC20TokenInfo,
    ) -> TokenBalance {
        todo!()
    }

    fn get_chain(&self) -> &'static Chain {
        &ZKSYNC
    }
}
