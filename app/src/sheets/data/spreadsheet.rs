use super::spreadsheet_manager::SpreadsheetManager;
use crate::sheets::{into::MyInto, ranges};

pub struct SpreadsheetUseCasesImpl<'s> {
    pub spreadsheet_manager: &'s SpreadsheetManager,
}

pub enum BalanceUpdateTarget {
    Binance,
    Kraken,
}

fn get_target_range(target: BalanceUpdateTarget) -> &'static str {
    match target {
        BalanceUpdateTarget::Binance => ranges::balances::binance::RW_AMOUNTS,
        BalanceUpdateTarget::Kraken => ranges::balances::kraken::RW_AMOUNTS,
    }
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
            .expect("Should have content, when getting token names, can't continue without it")
            .values
            .expect("Should have values when getting token names, can't continue without them")
            .my_into()
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
