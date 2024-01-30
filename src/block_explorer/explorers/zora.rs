use crate::constants::ETH_IN_WEI;
use crate::token::{Token, TokenBalance};
use crate::BlockExplorer;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    coin_balance: String,
}

pub struct ZoraExplorer;

impl BlockExplorer for ZoraExplorer {
    fn fetch_balance(&self, evm_address: &str) -> TokenBalance {
        let url = format!("https://explorer.zora.energy/api/v2/addresses/{evm_address}",);
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.coin_balance.parse::<f64>().unwrap() / (ETH_IN_WEI as f64);

        TokenBalance {
            token: self.get_native_token(),
            balance,
        }
    }

    fn get_native_token(&self) -> Token {
        Token::ETH
    }
}
