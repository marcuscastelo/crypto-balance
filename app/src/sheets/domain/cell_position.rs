use std::{num::ParseIntError, str::FromStr};

use thiserror::Error;

use super::{
    a1_notation::{
        generic_a1_notation_split, A1Notation, A1NotationParseError, A1NotationParts,
        FromA1Notation,
    },
    column::{parse_col, Column, ColumnParseError},
    row::Row,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPosition {
    pub col: Column,
    pub row: Row,
}

/// CellPosition operations

impl std::ops::Add<(u32, u32)> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: (u32, u32)) -> Self::Output {
        CellPosition {
            col: self.col + rhs.0,
            row: self.row + rhs.1,
        }
    }
}

impl std::ops::Sub<(u32, u32)> for CellPosition {
    type Output = CellPosition;

    fn sub(self, rhs: (u32, u32)) -> Self::Output {
        CellPosition {
            col: self.col - rhs.0,
            row: self.row - rhs.1,
        }
    }
}

impl std::ops::Add<CellPosition> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: CellPosition) -> Self::Output {
        CellPosition {
            col: self.col + rhs.col,
            row: self.row + rhs.row,
        }
    }
}

impl std::ops::Sub<CellPosition> for CellPosition {
    type Output = CellPosition;

    fn sub(self, rhs: CellPosition) -> Self::Output {
        CellPosition {
            col: self.col - rhs.col,
            row: self.row - rhs.row,
        }
    }
}

impl std::ops::Add<Row> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: Row) -> Self::Output {
        CellPosition {
            col: self.col,
            row: self.row + rhs,
        }
    }
}

impl std::ops::Sub<Row> for CellPosition {
    type Output = CellPosition;

    fn sub(self, rhs: Row) -> Self::Output {
        CellPosition {
            col: self.col,
            row: self.row - rhs,
        }
    }
}

impl std::ops::Add<Column> for CellPosition {
    type Output = CellPosition;

    fn add(self, rhs: Column) -> Self::Output {
        CellPosition {
            col: self.col + rhs,
            row: self.row,
        }
    }
}

impl std::ops::Sub<Column> for CellPosition {
    type Output = CellPosition;

    fn sub(self, rhs: Column) -> Self::Output {
        CellPosition {
            col: self.col - rhs,
            row: self.row,
        }
    }
}

/// Conversions: Others -> CellPosition

impl<C: Into<Column>, R: Into<Row>> From<(C, R)> for CellPosition {
    fn from((col, row): (C, R)) -> Self {
        CellPosition {
            col: col.into(),
            row: row.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CellPositionParseError {
    #[error("Error parsing column: {0}")]
    ColumnParseError(ColumnParseError),
    #[error("Error parsing row: {0}")]
    RowParseError(ParseIntError),
}

impl<R: Into<Row>> TryFrom<(char, R)> for CellPosition {
    type Error = CellPositionParseError;

    fn try_from((col, row): (char, R)) -> Result<Self, Self::Error> {
        Ok(CellPosition {
            col: col
                .try_into()
                .map_err(CellPositionParseError::ColumnParseError)?,
            row: row.into(),
        })
    }
}

impl TryFrom<&str> for CellPosition {
    type Error = CellPositionParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (col, row) = s.split_at(s.chars().position(char::is_numeric).unwrap());
        Ok(CellPosition {
            col: col
                .parse()
                .map_err(CellPositionParseError::ColumnParseError)?,
            row: row.parse().map_err(CellPositionParseError::RowParseError)?,
        })
    }
}

impl FromStr for CellPosition {
    type Err = CellPositionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

/// Conversions: CellPosition -> Others

impl From<CellPosition> for (u32, u32) {
    fn from(cell_position: CellPosition) -> Self {
        (cell_position.row.into(), cell_position.col.into())
    }
}

impl FromA1Notation for CellPosition {
    type Err = A1NotationParseError;

    fn from_a1_notation(a1_notation: &A1Notation) -> Result<Self, Self::Err> {
        let parts: A1NotationParts = generic_a1_notation_split(a1_notation);

        Ok(CellPosition {
            row: parts
                .start
                .as_str()
                .parse()
                .expect(format!("Error parsing row: {}", parts.start).as_str()),
            col: parse_col(&parts.end).map_err(A1NotationParseError::ColumnParseError)?,
        })
    }
}
