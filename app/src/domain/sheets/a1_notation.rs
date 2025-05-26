use std::fmt::Formatter;
use thiserror::Error;

use super::{cell_position::CellPosition, column::ColumnParseError};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub struct A1Notation(pub String);

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

#[derive(Debug, Error)]
pub enum A1NotationParseError {
    #[error("Error parsing column")]
    ColumnParseError,
    #[error("Error parsing row")]
    RowParseError,
}

impl From<String> for A1Notation {
    fn from(s: String) -> Self {
        A1Notation(s)
    }
}

pub trait FromA1Notation: Sized {
    type Err;

    fn from_a1_notation(a1_notation: &A1Notation) -> error_stack::Result<Self, Self::Err>;
}

impl ToA1Notation for CellPosition {
    fn to_a1_notation(&self, sheet_name: Option<&str>) -> A1Notation {
        match sheet_name {
            Some(sheet_name) => A1Notation(format!("'{}'!{}{}", sheet_name, self.col, self.row)),
            None => A1Notation(format!("{}{}", self.col, self.row)),
        }
    }
}

pub struct A1NotationParts {
    pub start: String,
    pub end: String,
    pub sheet_title: Option<String>,
}

pub fn generic_a1_notation_split(a1_notation: &A1Notation) -> A1NotationParts {
    let (sheet_title, local_a1_notation) = match a1_notation.0.find('!') {
        Some(index) => {
            let (sheet_title, local_a1_notation) = a1_notation.0.split_at(index);
            (
                Some(sheet_title.to_owned()),
                local_a1_notation.trim_start_matches('!').to_owned(),
            )
        }
        None => (None, a1_notation.0.clone()),
    };

    let (start, end) = match local_a1_notation.find(':') {
        Some(index) => {
            let (start, end) = local_a1_notation.split_at(index);
            (start.to_owned(), end.trim_start_matches(':').to_owned())
        }
        None => (local_a1_notation.clone(), local_a1_notation),
    };

    A1NotationParts {
        sheet_title,
        start,
        end,
    }
}
