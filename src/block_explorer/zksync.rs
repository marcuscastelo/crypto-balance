use super::BlockExplorer;
use crate::constants::ETH_IN_WEI;
use crate::token::{Token, TokenBalance};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    status: String,
    message: String,
    result: String,
}

pub struct ZkSyncExplorer;

impl BlockExplorer for ZkSyncExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!(
            "https://block-explorer-api.mainnet.zksync.io/api\
                ?module=account\
                &action=balance\
                &address={evm_address}"
        );
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.result.parse::<f64>().unwrap() / (ETH_IN_WEI as f64);

        TokenBalance {
            token: self.get_native_token(),
            balance,
        }
    }

    fn get_native_token(&self) -> Token {
        Token::ETH
    }
}
