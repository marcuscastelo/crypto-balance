use crate::prelude::*;
use crate::user_addresses::UserAddresses;
use google_sheets4::api::ValueRange;
use std::collections::HashMap;
use std::sync::Arc;

// TODO: decide where Airdrop Wallet will be (remove from here?)
pub struct UpdateAirdropWalletOnSheetsBalanceRoutine;

#[async_trait::async_trait]
impl Routine for UpdateAirdropWalletOnSheetsBalanceRoutine {
    async fn run(&self) {
        let user_addresses = UserAddresses::from_config(&CONFIG.blockchain);
        let sheet_title = "Balance per Chain - Airdrop Wallet";

        let evm_chain_balance_routines = EVM_CHAINS.values().map(|chain| async {
            (
                chain.name,
                routines::blockchain::FetchEvmChainBalancesRoutine
                    .run(chain, &CONFIG.blockchain.evm.address)
                    .await,
            )
        });

        let cosmos_chain_balance_routines = COSMOS_CHAINS.values().map(|chain| async {
            (
                chain.name,
                routines::blockchain::FetchCosmosChainBalancesRoutine
                    .run(
                        chain,
                        user_addresses
                            .get_addresses(chain)
                            .unwrap()
                            .first()
                            .unwrap(),
                    )
                    .await,
            )
        });

        let mut chain_balances: HashMap<&str, HashMap<Arc<Token>, TokenBalance>> = HashMap::new();

        chain_balances.extend(
            futures::future::join_all(cosmos_chain_balance_routines)
                .await
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );

        chain_balances.extend(
            futures::future::join_all(evm_chain_balance_routines)
                .await
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );

        println!("Chain balances: {:#?}", chain_balances);

        println!("Starting sheet manipulation...");

        let spreadsheet_manager = SpreadsheetManager::new(app_config::CONFIG.sheets.clone()).await;

        println!("Creating unique tokens...");

        // Create a set of unique token structs using their names as keys
        let mut unique_tokens: HashMap<String, Arc<Token>> = HashMap::new();
        for balances in chain_balances.values() {
            for token in balances.keys() {
                match token.as_ref() {
                    Token::Native(token_name) => {
                        unique_tokens.insert(token_name.to_string(), token.clone());
                    }
                    Token::ERC20(token_info) => {
                        unique_tokens.insert(token_info.token_symbol.to_string(), token.clone());
                    }
                    Token::IBC => todo!("IBC token not implemented yet"),
                }
            }
        }
        let mut unique_tokens = unique_tokens.into_iter().collect::<Vec<_>>();
        unique_tokens.sort_by(|a, b| a.0.cmp(&b.0));
        let unique_tokens = unique_tokens;

        println!("Writing token names...");
        let token_names = unique_tokens
            .iter()
            .map(|(_, token)| match token.as_ref() {
                Token::Native(token_name) => token_name.to_string(),
                Token::ERC20(token_info) => token_info.token_symbol.to_string(),
                Token::IBC => todo!("IBC token not implemented yet"),
            })
            .collect::<Vec<_>>();

        // Write the token names to the spreadsheet (B3:B1000)
        spreadsheet_manager
            .write_range(
                format!("'{}'!B3:B1000", sheet_title).as_str(),
                ValueRange::from_rows(token_names.as_ref()),
            )
            .await
            .expect("Should write token names to the spreadsheet");

        let mut chain_names = chain_balances.keys().cloned().collect::<Vec<_>>();
        chain_names.sort();
        let chain_names = chain_names;

        println!("Writing token names done!");

        let starting_col: Column = "C".try_into().unwrap();
        for (current_chain_idx, chain_name) in chain_names.iter().enumerate() {
            println!("Writing balances for {}", chain_name);

            let current_col = starting_col + current_chain_idx as u32;
            let chain_name_position = CellPosition {
                row: 2_usize.into(),
                col: current_col,
            };

            spreadsheet_manager
                .write_range(
                    chain_name_position
                        .to_a1_notation(Some(sheet_title))
                        .as_ref(),
                    ValueRange::from_str(chain_name),
                )
                .await
                .unwrap();

            let mut token_balances = Vec::with_capacity(unique_tokens.len());
            for (_, token) in &unique_tokens {
                token_balances.push(
                    chain_balances
                        .get(chain_name)
                        .unwrap_or_else(|| panic!("Chain {} should have balance", chain_name))
                        .get(token)
                        .map(|x| x.balance.to_string())
                        .unwrap_or("".to_owned()),
                );
            }

            let pivot: CellPosition = (current_col, 3_u32).into();

            let balances_range = CellRange {
                start: pivot.clone(),
                end: (pivot.col, pivot.row + token_balances.len()).into(),
            };

            println!("Writing to range: {:#?}", balances_range);
            spreadsheet_manager
                .write_range(
                    balances_range.to_a1_notation(Some(sheet_title)).as_ref(),
                    ValueRange::from_rows(
                        token_balances
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .as_ref(),
                    ),
                )
                .await
                .unwrap();

            println!("Writing balances for {} done!", chain_name);
            println!(
                "Written: {:?}",
                ValueRange::from_rows(
                    token_balances
                        .iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .as_ref(),
                )
            );
        }
        println!("Writing balances done!");
    }
}
