use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::blockchain::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use error_stack::{Result, ResultExt};

use self::block_explorer::explorer::FetchBalanceError;

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

async fn fetch_and_deserialize<T: DeserializeOwned>(url: &str) -> Result<T, FetchBalanceError> {
    let resp = reqwest::get(url)
        .await
        .change_context(FetchBalanceError::ReqwestError)
        .attach_printable("Failed to make GET request")
        .attach_printable_lazy(|| format!("URL: {}", url))?;
    let resp = resp
        .text()
        .await
        .change_context(FetchBalanceError::ReqwestError)
        .attach_printable("Failed to get response text")
        .attach_printable_lazy(|| format!("URL: {}", url))?;
    let resp = serde_json::from_str(resp.as_str())
        .change_context(FetchBalanceError::DataFormatError)
        .attach_printable("Failed to parse balance response as json")
        .attach_printable_lazy(|| format!("Response: {}", resp))?;
    Ok(resp)
}

async fn parse_balance_from_response(resp: FetchBalanceResponse) -> Result<f64, FetchBalanceError> {
    let balance = resp
        .result
        .parse::<f64>()
        .change_context(FetchBalanceError::DataFormatError)
        .attach_printable_lazy(|| format!("Result was not a float! Result: {}", resp.result))?;
    Ok(balance)
}

// TODO: change panic to error
#[async_trait]
impl BlockExplorer for EtherscanImplementation {
    async fn fetch_native_balance(
        &self,
        evm_address: &str,
    ) -> Result<TokenBalance, FetchBalanceError> {
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

        let resp = fetch_and_deserialize(&url).await?;
        let balance = parse_balance_from_response(resp).await? / WEI_CONVERSION;

        Ok(TokenBalance {
            symbol: self.get_chain().native_token.symbol(),
            balance,
        })
    }

    async fn fetch_erc20_balance(
        &self,
        evm_address: &str,
        token_info: ERC20TokenInfo,
    ) -> Result<TokenBalance, FetchBalanceError> {
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

        let resp = fetch_and_deserialize(&url).await?;
        let balance = parse_balance_from_response(resp).await? / WEI_CONVERSION;

        Ok(TokenBalance {
            symbol: token_info.token_symbol.into_string(),
            balance,
        })
    }

    async fn fetch_erc20_balances(
        &self,
        evm_address: &str,
    ) -> Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError> {
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

        let resp: FetchTokenTxResponse = fetch_and_deserialize(&url).await?;

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
                    .await
                    .attach_printable_lazy(|| {
                        format!("Failed to fetch balance for token: {:?}", token)
                    })?;

                // Wait for 0.25 seconds between each request to avoid rate limiting
                std::thread::sleep(std::time::Duration::from_millis(250));

                balances.insert(token.into(), balance);
            } else {
                unreachable!("Token should be ERC20 since we just converted it")
            }
        }

        Ok(balances)
    }

    fn get_chain(&self) -> &'static Chain {
        *self.chain
    }
}
