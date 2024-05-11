use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use crate::BlockExplorer;
use async_trait::async_trait;

use crate::blockchain::prelude::*;

use self::{block_explorer::explorer::FetchBalanceError, mintscan_responses::DelegationsResponse};

mod mintscan_responses {

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub struct BalanceResponse {
        pub denom: String,
        pub amount: String,
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub struct BalancesResponse {
        pub balances: Vec<BalanceResponse>,
        // pub pagination: serde_json::Value, // Ignored for now
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub struct DelegationResponse {
        pub balance: BalanceResponse,
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub struct DelegationsResponse {
        pub delegation_responses: Vec<DelegationResponse>,
        // pub pagination: serde_json::Value, // Ignored for now
    }
}

#[derive(Debug)]
pub struct Mintscan {
    pub lcd_url: &'static str,
    pub chain: LazyLock<&'static Chain>,
}

const BANK_PATH: &str = "cosmos/bank/v1beta1";
const STAKING_PATH: &str = "cosmos/staking/v1beta1";

// TODO: remove code duplication and return BalanceResponse instead of f64
impl Mintscan {
    async fn fetch_bank_balance(&self, cosmos_address: &str) -> f64 {
        let lcd_url = self.lcd_url;
        let url = format!("{lcd_url}/{BANK_PATH}/balances/{cosmos_address}",);
        let resp = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::ORIGIN, "https://www.mintscan.io")
            .header(reqwest::header::REFERER, "https://www.mintscan.io")
            .send()
            .await
            .expect("Should send request and receive response")
            .text() // TODO: change to json to avoid double parsing (also on other block explorers)
            .await
            .expect("Should receive response as text");

        let resp: mintscan_responses::BalancesResponse =
            serde_json::from_str(&resp).expect("Should parse response as JSON");
        let mut balance = resp
            .balances
            .iter()
            // Checks for uATOM, uTIA, INJ, etc. and not for factory/contract/ibc tokens // TODO: SUPPORT IBC
            .filter(|d| !d.denom.contains('/'))
            .map(|b| b.amount.parse::<f64>().unwrap())
            .reduce(|a, b| a + b)
            .unwrap_or(0f64);

        //TODO: move to other place
        balance /= match *self.get_chain().native_token {
            Token::Native(NativeTokenSymbol::ATOM) => ATOM_U_CONVERSION,
            Token::Native(NativeTokenSymbol::OSMO) => OSMO_U_CONVERSION,
            Token::Native(NativeTokenSymbol::TIA) => CELESTIA_U_CONVERSION,
            Token::Native(NativeTokenSymbol::INJ) => INJECTIVE_U_CONVERSION,
            _ => unreachable!("Unsupported token"),
        };

        balance
    }

    async fn fetch_staking_balance(&self, cosmos_address: &str) -> f64 {
        let lcd_url = self.lcd_url;
        let url = format!("{lcd_url}/{STAKING_PATH}/delegations/{cosmos_address}",);
        let resp = reqwest::Client::new()
            .get(url)
            .header(reqwest::header::ORIGIN, "https://www.mintscan.io")
            .header(reqwest::header::REFERER, "https://www.mintscan.io")
            .send()
            .await
            .expect("Should send request and receive response")
            .text() // TODO: change to json to avoid double parsing (also on other block explorers)
            .await
            .expect("Should receive response as text");

        let resp: DelegationsResponse =
            serde_json::from_str(&resp).expect("Should parse response as JSON");
        let mut balance = resp
            .delegation_responses
            .iter()
            // Checks for uATOM, uTIA, INJ, etc. and not for factory/contract/ibc tokens // TODO: SUPPORT IBC
            .filter(|d| !d.balance.denom.contains('/'))
            .map(|d| d.balance.amount.parse::<f64>().unwrap())
            .reduce(|a, b| a + b)
            .unwrap_or(0f64);

        //TODO: move to other place
        balance /= match *self.get_chain().native_token {
            Token::Native(NativeTokenSymbol::ATOM) => ATOM_U_CONVERSION,
            Token::Native(NativeTokenSymbol::OSMO) => OSMO_U_CONVERSION,
            Token::Native(NativeTokenSymbol::TIA) => CELESTIA_U_CONVERSION,
            Token::Native(NativeTokenSymbol::INJ) => INJECTIVE_U_CONVERSION,
            _ => unreachable!("Unsupported token"),
        };

        balance
    }
}

#[async_trait]
impl BlockExplorer for Mintscan {
    async fn fetch_native_balance(
        &self,
        cosmos_address: &str,
    ) -> Result<TokenBalance, FetchBalanceError> {
        let bank_balance = self.fetch_bank_balance(cosmos_address).await;
        let staking_balance = self.fetch_staking_balance(cosmos_address).await;

        Ok(TokenBalance {
            symbol: self.get_chain().native_token.symbol(),
            balance: bank_balance + staking_balance,
        })
    }

    async fn fetch_erc20_balance(
        &self,
        _evm_address: &str,
        _token_info: ERC20TokenInfo,
    ) -> Result<TokenBalance, FetchBalanceError> {
        todo!()
    }

    async fn fetch_erc20_balances(
        &self,
        _evm_address: &str,
    ) -> Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError> {
        todo!()
    }

    fn get_chain(&self) -> &'static Chain {
        &self.chain
    }
}
