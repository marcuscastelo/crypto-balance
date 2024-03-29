use async_trait::async_trait;

use crate::blockchain::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

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

#[derive(Debug)]
pub struct EtherscanImplementation {
    pub api_key: Box<str>,
    pub base_url: String,
    pub chain: LazyLock<&'static Chain>,
}

#[async_trait]
impl BlockExplorer for EtherscanImplementation {
    async fn fetch_native_balance(&self, evm_address: &str) -> TokenBalance {
        let api_key = self.api_key.as_ref();
        let base_url = self.base_url.as_str();
        let url = format!(
            "{base_url}\
                ?module=account\
                &action=balance\
                &address={evm_address}\
                &tag=latest\
                &apikey={api_key}"
        );
        let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = match resp.result.parse::<f64>() {
            Ok(balance) => balance / WEI_CONVERSION,
            Err(_) => {
                panic!("Error fetching balance: {:?}", resp);
            }
        };

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
        let api_key = self.api_key.as_ref();
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
        let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
        let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
        let balance = resp.result.parse::<f64>().unwrap() / WEI_CONVERSION;

        TokenBalance {
            token: Token::ERC20(token_info.clone()).into(),
            balance,
        }
    }

    async fn fetch_erc20_balances(&self, evm_address: &str) -> HashMap<Arc<Token>, TokenBalance> {
        // Step 1. Fetch all ERC20 token transfers for the given address
        // Step 2. For each token, fetch the balance of the token for the given address
        // Attention: wait for 0.25 seconds between each request to avoid rate limiting

        // TODO: Create functions for step 1 and step 2
        let api_key = self.api_key.as_ref();
        let base_url = self.base_url.as_str();
        let url: String = format!(
            "{base_url}\
            ?module=account\
            &action=tokentx\
            &address={evm_address}\
            &tag=latest&apikey={api_key}"
        );
        let resp = reqwest::get(url)
            .await
            .expect("Should make GET request")
            .text()
            .await
            .expect("Should get response text");
        let resp: FetchTokenTxResponse = serde_json::from_str(&resp).expect("Should parse JSON");

        let tokens: Vec<_> = resp
            .result
            .into_iter()
            .filter(|info| !token::spam_filter::check_spam(&info.token_symbol, &info.token_name))
            .map(Token::ERC20)
            .collect();

        let mut balances = HashMap::new();
        for token in tokens {
            if let Token::ERC20(token_info) = &token {
                if balances.contains_key(&token) {
                    continue; // Skip if we already fetched the balance for this token
                }

                let balance = self
                    .fetch_erc20_balance(evm_address, token_info.clone())
                    .await;

                // Wait for 0.25 seconds between each request to avoid rate limiting
                std::thread::sleep(std::time::Duration::from_millis(250));

                balances.insert(token.into(), balance);
            } else {
                unreachable!("Token should be ERC20 since we just converted it")
            }
        }

        balances
    }

    fn get_chain(&self) -> &'static Chain {
        *self.chain
    }
}
