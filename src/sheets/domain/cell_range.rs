use std::num::TryFromIntError;

use super::cell_position::CellPosition;

use google_sheets4::api::GridRange;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellPosition,
    pub end: CellPosition,
}

impl CellRange {
    pub fn row_count(&self) -> u32 {
        self.end.row.0 - self.start.row.0 + 1
    }

    pub fn column_count(&self) -> u32 {
        self.end.col.0 - self.start.col.0 + 1
    }
}

/// Conversions: Others -> CellRange

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CellRangeParseError {
    #[error("Invalid grid range: {0}")]
    InvalidGridRange(InvalidGridRangeKind),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidGridRangeKind {
    #[error("Missing {0}")]
    Missing(InvalidGridRangeTarget),
    #[error("Error while parsing {target} as an integer: {error} ")]
    TryFromIntError {
        target: InvalidGridRangeTarget,
        error: TryFromIntError,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidGridRangeTarget {
    #[error("Start column")]
    StartColumn,
    #[error("End column")]
    EndColumn,
    #[error("Start row")]
    StartRow,
    #[error("End row")]
    EndRow,
}

impl TryFrom<GridRange> for CellRange {
    type Error = CellRangeParseError;

    fn try_from(grid_range: GridRange) -> Result<Self, CellRangeParseError> {
        let start_column_index =
            grid_range
                .start_column_index
                .ok_or(CellRangeParseError::InvalidGridRange(
                    InvalidGridRangeKind::Missing(InvalidGridRangeTarget::StartColumn),
                ))?
                + 1;

        let end_column_index =
            grid_range
                .end_column_index
                .ok_or(CellRangeParseError::InvalidGridRange(
                    InvalidGridRangeKind::Missing(InvalidGridRangeTarget::EndColumn),
                ))?;

        let start_row_index =
            grid_range
                .start_row_index
                .ok_or(CellRangeParseError::InvalidGridRange(
                    InvalidGridRangeKind::Missing(InvalidGridRangeTarget::StartRow),
                ))?
                + 1;

        let end_row_index =
            grid_range
                .end_row_index
                .ok_or(CellRangeParseError::InvalidGridRange(
                    InvalidGridRangeKind::Missing(InvalidGridRangeTarget::EndRow),
                ))?;

        let start = CellPosition {
            row: u32::try_from(start_row_index)
                .map_err(|error| {
                    CellRangeParseError::InvalidGridRange(InvalidGridRangeKind::TryFromIntError {
                        target: InvalidGridRangeTarget::StartRow,
                        error,
                    })
                })?
                .into(),
            col: u32::try_from(start_column_index)
                .map_err(|error| {
                    CellRangeParseError::InvalidGridRange(InvalidGridRangeKind::TryFromIntError {
                        target: InvalidGridRangeTarget::StartColumn,
                        error,
                    })
                })?
                .into(),
        };

        let end = CellPosition {
            row: u32::try_from(end_row_index)
                .map_err(|error| {
                    CellRangeParseError::InvalidGridRange(InvalidGridRangeKind::TryFromIntError {
                        target: InvalidGridRangeTarget::EndRow,
                        error,
                    })
                })?
                .into(),
            col: u32::try_from(end_column_index)
                .map_err(|error| {
                    CellRangeParseError::InvalidGridRange(InvalidGridRangeKind::TryFromIntError {
                        target: InvalidGridRangeTarget::EndColumn,
                        error,
                    })
                })?
                .into(),
        };

        Ok(CellRange { start, end })
    }
}
