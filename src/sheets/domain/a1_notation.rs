use std::{fmt::Formatter, num::ParseIntError, ops::Deref, str::FromStr};
use thiserror::Error;

use super::{
    cell_position::CellPosition,
    cell_range::CellRange,
    column::{number_to_letters, ColumnParseError},
};

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

pub struct A1NotationParts {
    pub sheet_name: Option<String>,
    pub start: String,
    pub end: String,
}

pub fn generic_a1_notation_split(a1_notation: &A1Notation) -> A1NotationParts {
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
