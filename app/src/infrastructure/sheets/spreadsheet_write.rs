use crate::domain::sheets::a1_notation::A1Notation;
use crate::domain::sheets::a1_notation::ToA1Notation;
use crate::infrastructure::sheets::cell_range::CellRange;
use crate::infrastructure::sheets::spreadsheet_manager::SpreadsheetManager;
use crate::infrastructure::sheets::spreadsheet_manager::SpreadsheetManagerError;
use error_stack::{report, ResultExt};
use google_sheets4::api::ValueRange;
use tracing::instrument;

use super::value_range_factory::ValueRangeFactory;
pub trait SpreadsheetWrite {
    async fn write_value(
        &self,
        position_str: &A1Notation,
        value: &str,
    ) -> error_stack::Result<(), SpreadsheetManagerError>;

    async fn write_column(
        &self,
        range: &CellRange,
        values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError>;

    async fn write_named_cell(
        &self,
        name: &str,
        value: &str,
    ) -> error_stack::Result<(), SpreadsheetManagerError>;

    async fn write_named_column(
        &self,
        name: &str,
        values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError>;

    async fn write_named_two_columns(
        &self,
        name: &str,
        col1_values: &[String],
        col2_values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError>;
}

impl SpreadsheetWrite for SpreadsheetManager {
    #[instrument]
    async fn write_value(
        &self,
        position_str: &A1Notation,
        value: &str,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let value_range = ValueRange::from_single_cell(value);
        self.write_range(position_str, value_range).await
    }

    #[instrument]
    async fn write_column(
        &self,
        range: &CellRange,
        values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let value_range = ValueRange::from_single_column(values, range.row_count());
        self.write_range(
            &range.to_a1_notation(range.sheet_title.as_deref()),
            value_range,
        )
        .await
    }

    #[instrument]
    async fn write_named_cell(
        &self,
        name: &str,
        value: &str,
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 1 || cell_range.row_count() != 1 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a single cell, trying to write {} to it",
                        name, value
                    )
                });
        }

        let value_range = ValueRange::from_single_cell(value);
        return self.write_named_range(name, value_range).await;
    }

    #[instrument]
    async fn write_named_column(
        &self,
        name: &str,
        values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 1 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a single column, trying to write {:?} to it",
                        name, values
                    )
                });
        }

        let value_range = ValueRange::from_single_column(values, cell_range.row_count());
        return self.write_named_range(name, value_range).await;
    }

    #[instrument]
    async fn write_named_two_columns(
        &self,
        name: &str,
        col1_values: &[String],
        col2_values: &[String],
    ) -> error_stack::Result<(), SpreadsheetManagerError> {
        let grid_range = self.get_named_range(name).await?;

        let cell_range = CellRange::try_from_grid_range_with_sheet_manager(grid_range, self)
            .await
            .change_context(SpreadsheetManagerError::FailedToWriteRange)?;

        if cell_range.column_count() != 2 {
            return Err(report!(SpreadsheetManagerError::FailedToWriteRange))
                .attach_printable_lazy(|| {
                    format!(
                        "Named range {} is not a two columns, trying to write col1 {:?} and col2 {:?} to it",
                        name, col1_values, col2_values
                    )
                });
        }

        let value_range =
            ValueRange::from_two_columns(col1_values, col2_values, cell_range.row_count());
        return self.write_named_range(name, value_range).await;
    }
}
