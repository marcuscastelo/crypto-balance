use std::{
    fmt::Formatter,
    num::{ParseIntError, TryFromIntError},
    ops::Deref,
    str::FromStr,
};
use thiserror::Error;

use google_sheets4::api::GridRange;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub struct A1Notation(String);

impl std::fmt::Display for A1Notation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<A1Notation> for String {
    fn from(a1_notation: A1Notation) -> Self {
        a1_notation.0
    }
}

impl AsRef<str> for A1Notation {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub trait ToA1Notation {
    fn to_a1_notation(&self, sheet_name: Option<&str>) -> A1Notation;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum A1NotationParseError {
    #[error("Error parsing column: {0}")]
    ColumnParseError(ColumnParseError),
}

impl From<String> for A1Notation {
    fn from(s: String) -> Self {
        A1Notation(s)
    }
}

pub trait FromA1Notation: Sized {
    type Err;

    fn from_a1_notation(
        a1_notation: &A1Notation,
        sheet_id: Option<&i32>,
    ) -> Result<Self, Self::Err>;
}

#[derive(Debug, Clone)]
pub struct SheetIdentification {
    pub name: String,
}

// TODO: Forbid 0 or change to 0-indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column(u32);

/// All u32 operations are delegated to the inner value
///
/// This is done to allow for easy manipulation of the inner value

impl<T: Into<Column>> std::ops::Add<T> for Column {
    type Output = Column;

    fn add(self, rhs: T) -> Self::Output {
        Column(self.0 + rhs.into().0)
    }
}

impl<T: Into<Column>> std::ops::Sub<T> for Column {
    type Output = Column;

    fn sub(self, rhs: T) -> Self::Output {
        Column(self.0 - rhs.into().0)
    }
}

impl<T: Into<Row>> std::ops::Add<T> for Row {
    type Output = Row;

    fn add(self, rhs: T) -> Self::Output {
        Row(self.0 + rhs.into().0)
    }
}

impl<T: Into<Row>> std::ops::Sub<T> for Row {
    type Output = Row;

    fn sub(self, rhs: T) -> Self::Output {
        Row(self.0 - rhs.into().0)
    }
}

/// Display implementations

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Row {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Column {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Conversions: Others -> Row

impl From<u32> for Row {
    fn from(value: u32) -> Self {
        Row(value)
    }
}

impl From<usize> for Row {
    fn from(value: usize) -> Self {
        Row(value as u32)
    }
}

impl FromStr for Row {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Row(s.parse()?))
    }
}

/// Conversions: Row -> Others

impl From<Row> for u32 {
    fn from(row: Row) -> Self {
        row.0
    }
}

/// Conversions: Others -> Column

impl From<u32> for Column {
    fn from(value: u32) -> Self {
        Column(value)
    }
}

impl From<usize> for Column {
    fn from(value: usize) -> Self {
        Column(value as u32)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ColumnParseError {
    #[error("Non-alphabetic character in column")]
    NonAlphabeticCharacter,
}

impl TryFrom<char> for Column {
    type Error = ColumnParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        parse_col(value.to_string())
    }
}

impl FromStr for Column {
    type Err = ColumnParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_col(s)
    }
}

impl TryFrom<&str> for Column {
    type Error = ColumnParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_col(value)
    }
}

/// Conversions: Column -> Others

impl From<Column> for u32 {
    fn from(col: Column) -> Self {
        col.0
    }
}

impl From<Column> for String {
    fn from(col: Column) -> Self {
        number_to_letters(col.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellPosition {
    pub col: Column,
    pub row: Row,
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

impl<R: Into<Row>> TryFrom<(&str, R)> for CellPosition {
    type Error = CellPositionParseError;

    fn try_from((col, row): (&str, R)) -> Result<Self, Self::Error> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellPosition,
    pub end: CellPosition,
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

fn number_to_letters(number: u32) -> String {
    let mut number = number;
    let mut result = String::new();
    while number > 0 {
        let remainder = (number - 1) % 26;
        let letter = (remainder as u8 + b'A') as char;
        result.push(letter);
        number = (number - remainder) / 26;
    }
    result.chars().rev().collect()
}

fn parse_col<T: AsRef<str>>(col_str: T) -> Result<Column, ColumnParseError> {
    if col_str.as_ref().chars().any(|c| !c.is_ascii_alphabetic()) {
        return Err(ColumnParseError::NonAlphabeticCharacter);
    }

    let col_num = col_str
        .as_ref()
        .chars()
        .fold(0, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    Ok(Column(col_num))
}

impl ToA1Notation for CellPosition {
    fn to_a1_notation(&self, sheet_name: Option<&str>) -> A1Notation {
        let col_letter = number_to_letters(self.col.into());

        match sheet_name {
            Some(sheet_name) => A1Notation(format!("'{}'!{}{}", sheet_name, col_letter, self.row)),
            None => A1Notation(format!("{}{}", col_letter, self.row)),
        }
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

struct A1NotationParts {
    sheet_name: Option<String>,
    start: String,
    end: String,
}

fn generic_a1_notation_split(a1_notation: &A1Notation) -> A1NotationParts {
    let (sheet_name, local_a1_notation) = match a1_notation.0.find('!') {
        Some(index) => {
            let (sheet_name, local_a1_notation) = a1_notation.0.split_at(index);
            (
                Some(sheet_name.to_owned()),
                local_a1_notation.trim_start_matches('!').to_owned(),
            )
        }
        None => (None, a1_notation.0.clone()),
    };

    let (start, end) = match local_a1_notation.find(':') {
        Some(index) => {
            let (start, end) = local_a1_notation.split_at(index);
            (start, end.trim_start_matches(':'))
        }
        None => (local_a1_notation.as_str(), local_a1_notation.as_str()),
    };

    A1NotationParts {
        sheet_name,
        start: start.to_owned(),
        end: end.to_owned(),
    }
}

impl FromA1Notation for CellPosition {
    type Err = A1NotationParseError;

    fn from_a1_notation(
        a1_notation: &A1Notation,
        sheet_id: Option<&i32>,
    ) -> Result<Self, Self::Err> {
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

impl FromA1Notation for CellRange {
    type Err = A1NotationParseError;

    fn from_a1_notation(
        a1_notation: &A1Notation,
        sheet_id: Option<&i32>,
    ) -> Result<Self, Self::Err> {
        let parts = generic_a1_notation_split(a1_notation);

        Ok(CellRange {
            start: CellPosition::from_a1_notation(&A1Notation(parts.start), sheet_id)?,
            end: CellPosition::from_a1_notation(&A1Notation(parts.end), sheet_id)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Tests 0, negative, mixed case, non-alphabetic characters, etc.

    #[test]
    fn test_number_to_letter() {
        assert_eq!(number_to_letters(1), "A");
        assert_eq!(number_to_letters(26), "Z");
        assert_eq!(number_to_letters(27), "AA");
        assert_eq!(number_to_letters(52), "AZ");
        assert_eq!(number_to_letters(53), "BA");
        assert_eq!(number_to_letters(702), "ZZ");
        assert_eq!(number_to_letters(703), "AAA");
    }

    #[test]
    fn test_add_column() {
        assert_eq!(Column(1) + Column(1), Column(2));
        assert_eq!(Column(1) + 1u32, Column(2));
        assert_eq!(Column(0) + 1u32, Column(1));
    }

    #[test]
    fn test_sub_column() {
        assert_eq!(Column(2) - Column(1), Column(1));
        assert_eq!(Column(2) - 1u32, Column(1));
        assert_eq!(Column(1) - 1u32, Column(0));
    }

    #[test]
    fn test_add_row() {
        assert_eq!(Row(1) + Row(1), Row(2));
        assert_eq!(Row(1) + 1u32, Row(2));
        assert_eq!(Row(0) + 1u32, Row(1));
    }

    #[test]
    fn test_sub_row() {
        assert_eq!(Row(2) - Row(1), Row(1));
        assert_eq!(Row(2) - 1u32, Row(1));
        assert_eq!(Row(1) - 1u32, Row(0));
    }

    #[test]
    fn test_parse_col() {
        assert_eq!(parse_col("A").unwrap(), Column(1));
        assert_eq!(parse_col("Z").unwrap(), Column(26));
        assert_eq!(parse_col("AA").unwrap(), Column(27));
        assert_eq!(parse_col("AZ").unwrap(), Column(52));
        assert_eq!(parse_col("BA").unwrap(), Column(53));
        assert_eq!(parse_col("ZZ").unwrap(), Column(702));
        assert_eq!(parse_col("AAA").unwrap(), Column(703));
        assert_eq!(
            parse_col("A1").err().unwrap(),
            ColumnParseError::NonAlphabeticCharacter
        );
    }

    #[test]
    fn test_row_from_str() {
        assert_eq!(Row::from_str("1").unwrap(), Row(1));
        assert_eq!(Row::from_str("0").unwrap(), Row(0));
        assert_eq!(Row::from_str("100").unwrap(), Row(100));
        assert!(Row::from_str("1a").is_err());
    }

    #[test]
    fn test_cell_position_to_a1_notation() {
        let cell_position = CellPosition {
            col: Column(1),
            row: Row(1),
        };

        assert_eq!(
            cell_position.to_a1_notation(None),
            A1Notation("A1".to_owned())
        );
    }

    #[test]
    fn test_cell_position_ZZ100_to_a1_notation() {
        let cell_position = CellPosition {
            col: Column(702),
            row: Row(100),
        };

        assert_eq!(
            cell_position.to_a1_notation(None),
            A1Notation("ZZ100".to_owned())
        );
    }

    #[test]
    fn test_cell_position_to_a1_notation_with_sheet_name() {
        let cell_position = CellPosition {
            col: Column(1),
            row: Row(1),
        };

        assert_eq!(
            cell_position.to_a1_notation(Some("Sheet1")),
            A1Notation("'Sheet1'!A1".to_owned())
        );
    }

    #[test]
    fn test_cell_range_to_a1_notation() {
        let cell_range = CellRange {
            start: CellPosition {
                col: Column(1),
                row: Row(1),
            },
            end: CellPosition {
                col: Column(702),
                row: Row(100),
            },
        };

        assert_eq!(
            cell_range.to_a1_notation(None),
            A1Notation("A1:ZZ100".to_owned())
        );
    }

    #[test]
    fn test_cell_range_to_a1_notation_with_sheet_name() {
        let cell_range = CellRange {
            start: CellPosition {
                col: Column(1),
                row: Row(1),
            },
            end: CellPosition {
                col: Column(702),
                row: Row(100),
            },
        };

        assert_eq!(
            cell_range.to_a1_notation(Some("Sheet1")),
            A1Notation("'Sheet1'!A1:ZZ100".to_owned())
        );
    }

    // Tests that cell range can be parsed to A1Notation and back
    #[test]
    fn test_cell_range_from_a1_notation() {
        let cell_range = CellRange {
            start: CellPosition {
                col: Column(1),
                row: Row(1),
            },
            end: CellPosition {
                col: Column(26),
                row: Row(26),
            },
        };

        let a1_notation = cell_range.to_a1_notation(None);
        let parsed_cell_range = CellRange::from_a1_notation(&a1_notation, None).unwrap();

        assert_eq!(cell_range, parsed_cell_range);
    }
}
