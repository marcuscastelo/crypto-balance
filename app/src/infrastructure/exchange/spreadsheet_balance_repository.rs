use std::sync::Arc;

use error_stack::ResultExt;

use crate::application::exchange::use_cases::get_target_range;
use crate::domain::exchange::{BalanceRepository, BalanceRepositoryError, BalanceUpdateTarget};
use crate::domain::sheets::ranges;
use crate::infrastructure::sheets::spreadsheet_manager::SpreadsheetManager;
use crate::infrastructure::sheets::spreadsheet_read::SpreadsheetRead;
use crate::infrastructure::sheets::spreadsheet_write::SpreadsheetWrite;

pub struct SpreadsheetBalanceRepository {
    pub spreadsheet_manager: Arc<SpreadsheetManager>,
}

impl SpreadsheetBalanceRepository {
    pub fn new(spreadsheet_manager: Arc<SpreadsheetManager>) -> Self {
        Self {
            spreadsheet_manager,
        }
    }
}

#[async_trait::async_trait]
impl BalanceRepository for SpreadsheetBalanceRepository {
    async fn get_token_names(&self) -> error_stack::Result<Vec<String>, BalanceRepositoryError> {
        self.spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .change_context(BalanceRepositoryError::FetchTokenNamesError)
    }

    async fn update_balances(
        &self,
        target: BalanceUpdateTarget,
        balances: &[f64],
    ) -> error_stack::Result<(), BalanceRepositoryError> {
        let range = get_target_range(target);

        let balances_str = balances
            .iter()
            .map(|x| format!("${}", x))
            .collect::<Vec<_>>();

        self.spreadsheet_manager
            .write_named_column(range, &balances_str)
            .await
            .change_context(BalanceRepositoryError::UpdateBalancesError)
    }
}
