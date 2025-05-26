use crate::{
    domain::sheets::{
        a1_notation::{
            generic_a1_notation_split, A1Notation, A1NotationParseError, FromA1Notation,
            ToA1Notation,
        },
        cell_position::CellPosition,
        column::Column,
        row::Row,
    },
    infrastructure::sheets::spreadsheet_manager::SpreadsheetManager,
};

use error_stack::{report, ResultExt};
use google_sheets4::api::GridRange;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellPosition,
    pub end: CellPosition,
    pub sheet_title: Option<String>,
}

impl CellRange {
    pub fn row_count(&self) -> u32 {
        self.end.row.index() - self.start.row.index() + 1
    }

    pub fn column_count(&self) -> u32 {
        self.end.col.index() - self.start.col.index() + 1
    }

    pub fn with_sheet_title(&self, sheet_title: String) -> Self {
        Self {
            start: self.start.clone(),
            end: self.end.clone(),
            sheet_title: Some(sheet_title),
        }
    }

    fn convert_index(
        grid_range_index: Option<i32>,
        field: &str,
    ) -> error_stack::Result<u32, CellRangeParseError> {
        grid_range_index
            .ok_or_else(|| report!(CellRangeParseError::InvalidGridRange))
            .attach_printable_lazy(|| format!("Missing {field} index in grid range"))
            .and_then(|i| u32::try_from(i).change_context(CellRangeParseError::InvalidGridRange))
            .attach_printable_lazy(|| {
                format!("Invalid {field} index in grid rangem, could not convert to u32",)
            })
    }

    pub async fn try_from_grid_range_with_sheet_manager(
        grid_range: GridRange,
        spreadsheet_manager: &SpreadsheetManager,
    ) -> error_stack::Result<Self, CellRangeParseError> {
        let (start_column_index, end_column_index, start_row_index, end_row_index) = (
            CellRange::convert_index(grid_range.start_column_index, "start_column")?,
            CellRange::convert_index(grid_range.end_column_index, "end_column")? - 1, // End is exclusive, so we subtract 1
            CellRange::convert_index(grid_range.start_row_index, "start_row")?,
            CellRange::convert_index(grid_range.end_row_index, "end_row")? - 1, // End is exclusive, so we subtract 1
        );

        let (start, end) = (
            CellPosition {
                row: Row::from_index(start_row_index),
                col: Column::from_index(start_column_index),
            },
            CellPosition {
                row: Row::from_index(end_row_index),
                col: Column::from_index(end_column_index),
            },
        );

        let sheet_title = match grid_range.sheet_id {
            None => None,
            Some(sheet_id) => spreadsheet_manager
                .get_sheet_title(sheet_id)
                .await
                .map_err(|error| CellRangeParseError::GetSheetTitleError(error.to_string()))?
                .into(),
        };

        Ok(CellRange {
            start,
            end,
            sheet_title,
        })
    }
}

impl ToA1Notation for CellRange {
    fn to_a1_notation(&self, sheet_name: Option<&str>) -> A1Notation {
        let start = self.start.to_a1_notation(sheet_name);
        let end = self.end.to_a1_notation(sheet_name);

        let (_, start) = start.0.split_at(start.0.find('!').unwrap_or(0));
        let (_, end) = end.0.split_at(end.0.find('!').unwrap_or(0));

        match sheet_name {
            Some(sheet_name) => A1Notation(format!(
                "'{}'!{}:{}",
                sheet_name.trim_start_matches('\'').trim_end_matches('\''),
                start.trim_start_matches('!'),
                end.trim_start_matches('!')
            )),
            None => A1Notation(format!("{}:{}", start, end)),
        }
    }
}

impl FromA1Notation for CellRange {
    type Err = A1NotationParseError;

    fn from_a1_notation(a1_notation: &A1Notation) -> error_stack::Result<Self, Self::Err> {
        let parts = generic_a1_notation_split(a1_notation);

        Ok(CellRange {
            start: CellPosition::from_a1_notation(&A1Notation(parts.start))?,
            end: CellPosition::from_a1_notation(&A1Notation(parts.end))?,
            sheet_title: parts.sheet_title,
        })
    }
}

/// Conversions: Others -> CellRange

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CellRangeParseError {
    #[error("Missing or invalid grid range field")]
    InvalidGridRange,
    #[error("Error getting sheet title: {0}")]
    GetSheetTitleError(String),
}
