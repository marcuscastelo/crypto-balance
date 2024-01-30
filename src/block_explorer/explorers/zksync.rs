use std::collections::HashMap;

use crate::constants::ETH_IN_WEI;
use crate::token::{ERC20TokenInfo, NativeTokenName, Token, TokenBalance};

use crate::BlockExplorer;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchNativeBalanceResponse {
    status: String,
    message: String,
    result: String,
}

pub struct ZkSyncExplorer;

impl BlockExplorer for ZkSyncExplorer {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!(
            "https://block-explorer-api.mainnet.zksync.io/api\
                ?module=account\
                &action=balance\
                &address={evm_address}"
        );
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchNativeBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.result.parse::<f64>().unwrap() / (ETH_IN_WEI as f64);

        TokenBalance {
            token: self.get_native_token(),
            balance,
        }
    }

    fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Token, TokenBalance> {
        todo!()
    }

    fn fetch_erc20_balance(&self, evm_address: &str, token_info: &ERC20TokenInfo) -> TokenBalance {
        todo!()
    }

    fn get_native_token(&self) -> Token {
        Token::Native(NativeTokenName::ETH)
    }
}
