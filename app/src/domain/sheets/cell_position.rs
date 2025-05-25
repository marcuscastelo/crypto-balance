use error_stack::ResultExt;

use super::{
    a1_notation::{
        generic_a1_notation_split, A1Notation, A1NotationParseError, A1NotationParts,
        FromA1Notation,
    },
    column::{parse_col, Column},
    row::Row,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPosition {
    pub col: Column,
    pub row: Row,
}

/// CellPosition operations

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

/// Conversions: CellPosition -> Others
impl FromA1Notation for CellPosition {
    type Err = A1NotationParseError;

    fn from_a1_notation(a1_notation: &A1Notation) -> error_stack::Result<Self, Self::Err> {
        let parts: A1NotationParts = generic_a1_notation_split(a1_notation);

        Ok(CellPosition {
            row: parts
                .start
                .as_str()
                .parse::<Row>()
                .change_context(A1NotationParseError::RowParseError)?,
            col: parse_col(&parts.end).map_err(A1NotationParseError::ColumnParseError)?,
        })
    }
}
