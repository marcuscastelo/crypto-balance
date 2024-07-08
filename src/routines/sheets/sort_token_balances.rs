// pub use crate::prelude::*;

// use super::SheetsGetTokenNamesRoutine;

// pub struct SortTokenBalancesForSheets {
//     token_balances: Vec<TokenBalance>,
// }

// impl SortTokenBalancesForSheets {
//     pub fn new(token_balances: Vec<TokenBalance>) -> Self {
//         Self { token_balances }
//     }
// }

// #[async_trait::async_trait]
// impl Routine<Vec<TokenBalance>> for SortTokenBalancesForSheets {
//     async fn run(&self) -> Vec<TokenBalance> {
//         let token_names = SheetsGetTokenNamesRoutine.run().await;

//         let mut token_balances = self.token_balances.clone();

//         // Sort by token names (i.e. by the order in the spreadsheet)
//         token_balances.sort_by(|a, b| {
//             token_names
//                 .iter()
//                 .position(|name| name == &a.symbol)
//                 .cmp(&token_names.iter().position(|name| name == &b.symbol))
//         });

//         token_balances
//     }
// }
