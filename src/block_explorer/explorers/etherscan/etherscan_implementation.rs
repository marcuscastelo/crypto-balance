use std::collections::HashMap;

use crate::constants::ETH_IN_WEI;
use crate::token::{ERC20TokenInfo, Token, TokenBalance};

use crate::block_explorer::explorer::BlockExplorer;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    status: String,
    message: String,
    result: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchTokenTxResponse {
    status: String,
    message: String,
    result: Vec<ERC20TokenInfo>,
}

pub struct EtherscanImplementation {
    pub network_name: String,
    pub api_key: String,
    pub base_url: String,
    pub native_token: Token,
}

impl BlockExplorer for EtherscanImplementation {
    fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let api_key = self.api_key.as_str();
        let base_url = self.base_url.as_str();
        let url = format!(
            "{base_url}\
                ?module=account\
                &action=balance\
                &address={evm_address}\
                &tag=latest\
                &apikey={api_key}"
        );
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = match resp.result.parse::<f64>() {
            Ok(balance) => balance / (ETH_IN_WEI as f64),
            Err(_) => {
                panic!("Error fetching balance: {:?}", resp);
            }
        };

        TokenBalance {
            token: self.get_native_token(),
            balance,
        }
    }

    fn fetch_erc20_balance(&self, evm_address: &str, token_info: &ERC20TokenInfo) -> TokenBalance {
        let api_key = self.api_key.as_str();
        let base_url = self.base_url.as_str();
        let contract_address = &token_info.contract_address;
        let url = format!(
            "{base_url}\
                ?module=account\
                &action=tokenbalance\
                &contractaddress={contract_address}\
                &address={evm_address}\
                &tag=latest&apikey={api_key}"
        );
        let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.result.parse::<f64>().unwrap() / (ETH_IN_WEI as f64);

        TokenBalance {
            token: Token::ERC20(token_info.clone()),
            balance,
        }
    }

    fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Token, TokenBalance> {
        // Step 1. Fetch all ERC20 token transfers for the given address
        // Step 2. For each token, fetch the balance of the token for the given address
        // Attention: wait for 0.25 seconds between each request to avoid rate limiting

        // TODO: Create functions for step 1 and step 2
        let api_key = self.api_key.as_str();
        let base_url = self.base_url.as_str();
        let url: String = format!(
            "{base_url}\
            ?module=account\
            &action=tokentx\
            &address={evm_address}\
            &tag=latest&apikey={api_key}"
        );
        let resp = reqwest::blocking::get(url)
            .expect("Should make GET request")
            .text()
            .expect("Should get response text");
        let resp: FetchTokenTxResponse = serde_json::from_str(&resp).expect("Should parse JSON");

        let tokens: Vec<_> = resp.result.into_iter().map(Token::ERC20).collect();

        let mut balances = HashMap::new();
        for token in tokens {
            if let Token::ERC20(token_info) = &token {
                let balance = self.fetch_erc20_balance(evm_address, token_info);
                balances.insert(token, balance);
            } else {
                unreachable!("Token should be ERC20 since we just converted it")
            }
        }

        balances
    }

    fn get_native_token(&self) -> Token {
        self.native_token.clone()
    }
}
