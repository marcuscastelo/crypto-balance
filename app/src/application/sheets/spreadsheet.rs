use crate::application::exchange::use_cases::{get_target_range, BalanceUpdateTarget};
use crate::domain::sheets::ranges;
use crate::infrastructure::sheets::spreadsheet_manager::SpreadsheetManager;
use crate::infrastructure::sheets::spreadsheet_read::SpreadsheetRead;
use crate::infrastructure::sheets::spreadsheet_write::SpreadsheetWrite;

pub struct SpreadsheetUseCasesImpl<'s> {
    pub spreadsheet_manager: &'s SpreadsheetManager,
}

impl<'s> SpreadsheetUseCasesImpl<'s> {
    pub fn new(spreadsheet_manager: &'s SpreadsheetManager) -> Self {
        Self {
            spreadsheet_manager,
        }
    }

    pub async fn get_token_names_from_spreadsheet(&self) -> Vec<String> {
        self.spreadsheet_manager
            .read_named_range(ranges::tokens::RO_NAMES)
            .await
            .expect("Should read token names from the spreadsheet")
    }

    pub async fn update_balances_on_spreadsheet(
        &self,
        target: BalanceUpdateTarget,
        balances: &[f64],
    ) {
        let range = get_target_range(target);

        let balances_str = balances
            .iter()
            .map(|x| format!("${}", x))
            .collect::<Vec<_>>();

        self.spreadsheet_manager
            .write_named_column(range, &balances_str)
            .await
            .expect("Should write balances to the spreadsheet");
    }
}
