use error_stack::{report, ResultExt};
use tracing::instrument;

use crate::domain::sheets::a1_notation::ToA1Notation;

use crate::adapters::sheets::cell_range::CellRange;

use super::{
    flatten_double_vec::FlattenDoubleVec,
    spreadsheet_manager::{SpreadsheetManager, SpreadsheetManagerError},
};

pub trait SpreadsheetRead {
    fn read_range(
        &self,
        range: &str,
    ) -> impl std::future::Future<Output = error_stack::Result<Vec<String>, SpreadsheetManagerError>>
           + Send;
    fn read_named_range(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = error_stack::Result<Vec<String>, SpreadsheetManagerError>>
           + Send;
}

impl SpreadsheetRead for SpreadsheetManager {
    #[instrument]
    async fn read_range(
        &self,
        range: &str,
    ) -> error_stack::Result<Vec<String>, SpreadsheetManagerError> {
        let response = self
            .hub
            .spreadsheets()
            .values_get(&self.config.spreadsheet_id, range)
            .doit()
            .await
            .change_context(SpreadsheetManagerError::FailedToFetchRange)?;

        let value_range = response.1;

        let values = value_range
            .values
            .ok_or(report!(SpreadsheetManagerError::FailedToFetchRange))
            .attach_printable_lazy(|| format!("Failed to fetch values for range {}", range))?
            .flatten_double_vec();

        Ok(values)
    }

    #[instrument]
    async fn read_named_range(
        &self,
        name: &str,
    ) -> error_stack::Result<Vec<String>, SpreadsheetManagerError> {
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
