use std::{fmt::Formatter, ops::Deref, str::FromStr};

use thiserror::Error;

// TODO: Forbid 0 or change to 0-indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column(pub u32);

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

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Column {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
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

pub fn parse_col<T: AsRef<str>>(col_str: T) -> Result<Column, ColumnParseError> {
    if col_str.as_ref().chars().any(|c| !c.is_ascii_alphabetic()) {
        return Err(ColumnParseError::NonAlphabeticCharacter);
    }

    let col_num = col_str
        .as_ref()
        .chars()
        .fold(0, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    Ok(Column(col_num))
}

pub fn number_to_letters(number: u32) -> String {
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
