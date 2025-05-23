use std::{collections::HashMap, sync::Arc};

use error_stack::ResultExt;
use google_sheets4::api::ValueRange;
use regex::Regex;
use tracing::instrument;

use crate::{
    config::app_config::{self, CONFIG},
    prelude::{
        block_explorer::explorer::FetchBalanceError, Chain, Token, TokenBalance, ARBITRUM,
        OPTIMISM, POLYGON,
    },
    sheets::{
        data::spreadsheet_manager::SpreadsheetManager,
        domain::{
            a1_notation::ToA1Notation, cell_position::CellPosition, cell_range::CellRange,
            column::Column, row::Row,
        },
        into::MyInto,
        ranges::{self, balances::hold},
        value_range_factory::ValueRangeFactory,
    },
};

use super::routine::{Routine, RoutineError};

#[derive(Debug)]
pub struct UpdateHoldBalanceOnSheetsRoutine;

struct TokenBalanceProcessor;

impl TokenBalanceProcessor {
    fn translate_aave_supply_token(&self, token: &str) -> (String, bool) {
        let aave_regex = Regex::new(r"^a(?:Opt)?(\w+)$").unwrap();
        match aave_regex.captures(token) {
            None => return (token.to_owned(), false),
            Some(captures) => return (captures.get(1).unwrap().as_str().to_owned(), true),
        }
    }

    fn translate_token_to_sheets_name(&self, token: &str) -> String {
        match token {
            "WBTC" => "BTC".to_owned(),
            _ => token.to_owned(),
        }
    }

    fn process_token_balance(&self, token: &str, balance: f64) -> TokenBalance<String> {
        let (translated_symbol, _) = self.translate_aave_supply_token(token);

        let mul = match translated_symbol.as_str() {
            "WBTC" => 1e10,
            _ => 1f64,
        };

        let translated_symbol = self.translate_token_to_sheets_name(&translated_symbol);

        TokenBalance::<String> {
            symbol: translated_symbol,
            balance: balance * mul,
        }
    }
}

impl UpdateHoldBalanceOnSheetsRoutine {
    #[instrument]
    pub async fn fetch_all_evm_balances(
        &self,
        chain: &Chain,
        evm_address: &str,
    ) -> error_stack::Result<HashMap<Arc<Token>, TokenBalance>, FetchBalanceError> {
        tracing::info!("Fetching balance for {}", chain.name);

        tracing::info!("Fetching native balance for {}", chain.name);
        let native_balance = chain
            .explorer
            .fetch_native_balance(evm_address)
            .await
            .attach_printable_lazy(|| {
                format!(
                    "Failed to fetch native balance for chain '{}', address '{}'",
                    chain.name, evm_address
                )
            })?;

        tracing::info!("Fetching ERC20 balances for {}", chain.name);
        let erc20_balances = chain
            .explorer
            .fetch_erc20_balances(evm_address)
            .await
            .attach_printable_lazy(|| {
                format!(
                    "Failed to fetch ERC20 balances for chain '{}', address '{}'",
                    chain.name, evm_address
                )
            })?;

        tracing::info!("Merging balances for {}", chain.name);
        let mut balances = erc20_balances;
        balances.insert(chain.native_token.to_owned(), native_balance);

        tracing::info!("Balances fetched for {}", chain.name);

        // Remove zero balances
        balances.retain(|_, balance| balance.balance > 0.0);
        Ok(balances)
    }

    #[instrument]
    async fn fetch_balance_hold(&self, chain: &Chain) -> HashMap<Arc<Token>, TokenBalance<String>> {
        self.fetch_all_evm_balances(chain, &CONFIG.blockchain.hold.evm.address)
            .await
            .expect(format!("Should fetch '{}' chain balances for hold", chain.name).as_str())
    }

    #[instrument]
    async fn fetch_balance_hold_sc(
        &self,
        chain: &Chain,
    ) -> HashMap<Arc<Token>, TokenBalance<String>> {
        self.fetch_all_evm_balances(chain, &CONFIG.blockchain.hold_sc.evm.address)
            .await
            .expect(format!("Should fetch '{}' chain balances for hold_sc", chain.name).as_str())
    }

    #[instrument]
    async fn create_spreadsheet_manager(&self) -> SpreadsheetManager {
        SpreadsheetManager::new(crate::config::app_config::CONFIG.sheets.clone()).await
    }

    #[instrument]
    async fn get_token_names_from_spreadsheet(&self) -> Vec<String> {
        let spreadsheet_manager = self.create_spreadsheet_manager().await;

        spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should have content")
            .values
            .expect("Should have values")
            .my_into()
    }
}

#[async_trait::async_trait]
impl Routine for UpdateHoldBalanceOnSheetsRoutine {
    fn name(&self) -> &'static str {
        "UpdateHoldBalanceOnSheetsRoutine"
    }

    #[instrument(skip(self))]
    async fn run(&self) -> error_stack::Result<(), RoutineError> {
        let chains = vec![&POLYGON, &OPTIMISM, &ARBITRUM];

        //Parallelize fetching balances from multiple chains
        let tasks = chains.iter().map(|chain| async move {
            let hold_balances_raw = self.fetch_balance_hold(chain).await;
            let hold_sc_balances_raw = self.fetch_balance_hold_sc(chain).await;

            let hold_balances_compressed = hold_balances_raw.into_iter().fold(
                HashMap::new(),
                |mut acc, (_, token_balance)| {
                    let processed_token_balance = TokenBalanceProcessor
                        .process_token_balance(&token_balance.symbol, token_balance.balance);

                    let acc_entry = acc.entry(processed_token_balance.symbol.clone()).or_insert(
                        TokenBalance::<String> {
                            symbol: processed_token_balance.symbol,
                            balance: 0f64,
                        },
                    );

                    acc_entry.balance += processed_token_balance.balance;
                    tracing::info!("{}: {}", acc_entry.symbol, acc_entry.balance);
                    acc
                },
            );

            let hold_sc_balances_compressed = hold_sc_balances_raw.into_iter().fold(
                HashMap::new(),
                |mut acc, (_, token_balance)| {
                    let processed_token_balance = TokenBalanceProcessor
                        .process_token_balance(&token_balance.symbol, token_balance.balance);

                    let acc_entry = acc.entry(processed_token_balance.symbol.clone()).or_insert(
                        TokenBalance::<String> {
                            symbol: processed_token_balance.symbol,
                            balance: 0f64,
                        },
                    );

                    acc_entry.balance += processed_token_balance.balance;
                    tracing::info!("{}: {}", acc_entry.symbol, acc_entry.balance);
                    acc
                },
            );

            (
                chain.name,
                (hold_balances_compressed, hold_sc_balances_compressed),
            )
        });

        let tasks_results = futures::future::join_all(tasks).await;

        let hashmaps = tasks_results.into_iter().collect::<HashMap<_, _>>();

        tracing::info!("Chains scanned: {:?}", hashmaps.keys().collect::<Vec<_>>());

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        let named_range = spreadsheet_manager
            .get_named_range(ranges::balances::hold::RW_DATA)
            .await
            .expect(
                format!(
                    "Should get named range '{}'",
                    ranges::balances::hold::RW_DATA
                )
                .as_str(),
            );

        let cell_range =
            CellRange::try_from_grid_range_with_sheet_manager(named_range, &spreadsheet_manager)
                .await
                .expect("Named range parsing error");

        let mut chain_title_cell = cell_range.start;

        for chain in chains {
            tracing::info!("Balances for '{}'", chain.name);
            spreadsheet_manager
                .write_value(
                    &chain_title_cell.to_a1_notation("Balance - Trezor HOLD".into()),
                    chain.name,
                )
                .await
                .expect("Should write chain title");

            let chain_title_col: Column = chain_title_cell.col;
            let chain_title_row = chain_title_cell.row;

            let wallet_hold_title_cell = chain_title_cell.clone() + Row(1u32);
            let wallet_hold_title_cell_a1 =
                wallet_hold_title_cell.to_a1_notation("Balance - Trezor HOLD".into());
            let wallet_hold_sc_title_cell = wallet_hold_title_cell + Column(1u32);

            spreadsheet_manager
                .write_value(&wallet_hold_title_cell_a1, "Hold")
                .await
                .expect("Should write wallet hold title");

            spreadsheet_manager
                .write_value(
                    &wallet_hold_sc_title_cell.to_a1_notation("Balance - Trezor HOLD".into()),
                    "SC",
                )
                .await
                .expect("Should write wallet hold sc title");

            let (hold_balances, hold_sc_balances) = hashmaps
                .get(chain.name)
                .expect(format!("Should get '{}' chain balances", chain.name).as_str());

            let token_names = self.get_token_names_from_spreadsheet().await;

            let hold_col = chain_title_col;
            let hold_sc_col: Column = chain_title_col + Column(1u32);

            let tokens_start_row = chain_title_row + Row(1u32);
            let tokens_end_row = cell_range.end.row;

            let hold_balances_range = CellRange {
                start: CellPosition {
                    col: hold_col,
                    row: tokens_start_row,
                },
                end: CellPosition {
                    col: hold_col,
                    row: tokens_end_row,
                },
                sheet_title: Some("Balance - Trezor HOLD".into()),
            };

            let hold_sc_balances_range = CellRange {
                start: CellPosition {
                    col: hold_sc_col,
                    row: tokens_start_row,
                },
                end: CellPosition {
                    col: hold_sc_col,
                    row: tokens_end_row,
                },
                sheet_title: Some("Balance - Trezor HOLD".into()),
            };

            tracing::info!("  Hold balances:");
            let hold_tokens_in_order = token_names.iter().fold(Vec::new(), |mut acc, token| {
                let token_balance = match hold_balances.get(token) {
                    Some(token_balance) => token_balance.balance.to_string(),
                    None => "".to_owned(),
                };

                acc.push(token_balance);
                acc
            });

            let hold_sc_tokens_in_order = token_names.iter().fold(Vec::new(), |mut acc, token| {
                let token_balance = match hold_sc_balances.get(token) {
                    Some(token_balance) => token_balance.balance.to_string(),
                    None => "".to_owned(),
                };

                acc.push(token_balance);
                acc
            });

            tracing::info!("  Hold balances: {:?}", hold_tokens_in_order);
            spreadsheet_manager
                .write_column(&hold_balances_range, hold_tokens_in_order.as_slice())
                .await
                .expect("Should write hold balances");

            tracing::info!("  SC balances: {:?}", hold_tokens_in_order);
            spreadsheet_manager
                .write_column(&hold_sc_balances_range, hold_sc_tokens_in_order.as_slice())
                .await
                .expect("Should write hold sc balances");

            chain_title_cell = chain_title_cell + Column(2u32);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_aave_supply_token() {
        assert_eq!(
            TokenBalanceProcessor.translate_aave_supply_token("aUSDC"),
            ("USDC".to_owned(), true)
        );
        assert_eq!(
            TokenBalanceProcessor.translate_aave_supply_token("aOptUSDC"),
            ("USDC".to_owned(), true)
        );
        assert_eq!(
            TokenBalanceProcessor.translate_aave_supply_token("aOptBTC"),
            ("BTC".to_owned(), true)
        );
        assert_eq!(
            TokenBalanceProcessor.translate_aave_supply_token("USDT"),
            ("USDT".to_owned(), false)
        );
        assert_eq!(
            TokenBalanceProcessor.translate_aave_supply_token("BTC"),
            ("BTC".to_owned(), false)
        );
    }
}
