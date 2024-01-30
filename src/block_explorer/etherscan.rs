use super::BlockExplorer;
use crate::app_config::CONFIG;
use crate::constants::ETH_IN_WEI;
use crate::token::{Token, TokenBalance};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    status: String,
    message: String,
    result: String,
}

pub struct Etherscan;

impl BlockExplorer for Etherscan {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance {
        let api_key = &CONFIG.etherscan_api_key;
        let url = format!(
            "https://api.etherscan.io/api\
                ?module=account\
                &action=balance\
                &address={evm_address}\
                &tag=latest\
                &apikey={api_key}"
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
