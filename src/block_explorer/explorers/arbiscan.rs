use std::collections::HashMap;

use crate::app_config::CONFIG;
use crate::constants::ETH_IN_WEI;
use crate::token::{ERC20TokenInfo, NativeTokenName, Token, TokenBalance};
use crate::BlockExplorer;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchNativeBalanceResponse {
    status: String,
    message: String,
    result: String,
}

pub struct Arbiscan;

impl BlockExplorer for Arbiscan {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let api_key = &CONFIG.arbiscan_api_key;
        let url = format!(
            "https://api.arbiscan.io/api\
                ?module=account\
                &action=balance\
                &address={evm_address}\
                &tag=latest\
                &apikey={api_key}"
        );
        let resp = reqwest::blocking::get(url)
            .expect("Should make GET request")
            .text()
            .expect("Should get response text");
        let resp: FetchNativeBalanceResponse =
            serde_json::from_str(&resp).expect("Should parse JSON");
        let balance = match resp.result.parse::<f64>() {
            Ok(balance) => balance / (ETH_IN_WEI as f64),
            Err(_) => panic!("Could not parse balance, response: {:?}", resp),
        };

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
