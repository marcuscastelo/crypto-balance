use error_stack::{report, ResultExt};
use google_sheets4::api::ValueRange;
use tracing::instrument;

use crate::sheets::domain::{a1_notation::ToA1Notation, cell_range::CellRange};

use super::spreadsheet_manager::{SpreadsheetManager, SpreadsheetManagerError};

pub trait SpreadsheetRead {
    async fn read_range(
        &self,
        range: &str,
    ) -> error_stack::Result<ValueRange, SpreadsheetManagerError>;
    async fn read_named_range(
        &self,
        name: &str,
    ) -> error_stack::Result<ValueRange, SpreadsheetManagerError>;
}

impl SpreadsheetRead for SpreadsheetManager {
    #[instrument]
    async fn read_range(
        &self,
        range: &str,
    ) -> error_stack::Result<ValueRange, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .values_get(&self.config.spreadsheet_id, range)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchRange)?;

        let value_range = response.1;
        Ok(value_range)
    }

    #[instrument]
    async fn read_named_range(
        &self,
        name: &str,
    ) -> error_stack::Result<ValueRange, SpreadsheetManagerError> {
        let named_range = self.get_named_range(name).await?;
        let sheet_title = self
            .get_sheet_title(
                named_range
                    .sheet_id
                    .ok_or(report!(SpreadsheetManagerError::FailedToFetchSheetTitle))?,
            )
            .await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(named_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchNamedRange(
                "cell range conversion failed",
            ))?;

        self.read_range(
            cell_range
                .to_a1_notation(Some(sheet_title.as_str()))
                .as_ref(),
        )
        .await
    }
}
