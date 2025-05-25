use core::panic;
use std::{fmt::Formatter, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column(u32);

impl Column {
    pub fn new(value: u32) -> Self {
        if value == 0 {
            panic!("Column number cannot be zero");
        }
        Column(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::ops::Add for Column {
    type Output = Column;

    fn add(self, rhs: Column) -> Self::Output {
        Column(
            self.0
                .checked_add(rhs.0)
                .expect("attempt to add with overflow"),
        )
    }
}

impl std::ops::Sub for Column {
    type Output = Column;

    fn sub(self, rhs: Column) -> Self::Output {
        Column(
            self.0
                .checked_sub(rhs.0)
                .expect("attempt to subtract with overflow"),
        )
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", number_to_letters(self.0))
    }
}

impl std::fmt::Debug for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Show both the numeric and letter representation
        write!(f, "Column(u32: {}, letters: {})", self.0, self)
    }
}

/// Conversions: Others -> Column

impl From<u32> for Column {
    fn from(value: u32) -> Self {
        Column(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ColumnParseError {
    #[error("Non-alphabetic character in column")]
    NonAlphabeticCharacter,
}

impl FromStr for Column {
    type Err = ColumnParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_col(s)
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
        .map(|c| c.to_ascii_uppercase())
        .fold(0, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    Ok(Column(col_num))
}

fn number_to_letters(number: u32) -> String {
    if number == 0 {
        panic!("Column number cannot be zero");
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_addition() {
        let col1 = Column(5);
        let col2 = Column(3);
        assert_eq!(col1 + col2, Column(8));
    }

    #[test]
    fn test_column_subtraction() {
        let col1 = Column(5);
        let col2 = Column(3);
        assert_eq!(col1 - col2, Column(2));
    }

    #[test]
    fn test_column_display_a() {
        let col = Column(1);
        assert_eq!(col.to_string(), "A");
    }

    #[test]
    fn test_column_display_z() {
        let col = Column(26);
        assert_eq!(col.to_string(), "Z");
    }

    #[test]
    fn test_column_display_aa() {
        let col = Column(26 * 1 + 1);
        assert_eq!(col.to_string(), "AA");
    }

    #[test]
    fn test_column_display_ab() {
        let col = Column(26 * 1 + 2);
        assert_eq!(col.to_string(), "AB");
    }

    #[test]
    fn test_column_display_az() {
        let col = Column(26 * 1 + 26);
        assert_eq!(col.to_string(), "AZ");
    }

    #[test]
    fn test_column_display_ba() {
        let col = Column(26 * 2 + 1);
        assert_eq!(col.to_string(), "BA");
    }

    #[test]
    fn test_column_display_za() {
        let col = Column(26 * 26 + 1);
        assert_eq!(col.to_string(), "ZA");
    }

    #[test]
    fn test_column_display_zza() {
        let col = Column(26 * 26 * 26 + 26 * 26 + 1);
        assert_eq!(col.to_string(), "ZZA");
    }

    #[test]
    fn test_column_display_zzy() {
        let col = Column(26 * 26 * 26 + 26 * 26 + 25);
        assert_eq!(col.to_string(), "ZZY");
    }

    #[test]
    fn test_column_from_u32() {
        let col: Column = 5.into();
        assert_eq!(col, Column(5));
    }

    #[test]
    fn test_column_from_str_lower() {
        let col: Column = "a".parse().unwrap();
        assert_eq!(col, Column(1));
    }

    #[test]
    fn test_column_from_str_upper() {
        let col: Column = "A".parse().unwrap();
        assert_eq!(col, Column(1));
    }

    #[test]
    fn test_column_to_u32() {
        let col = Column(5);
        let value: u32 = col.into();
        assert_eq!(value, 5);
    }

    #[test]
    fn test_column_from_str_error() {
        let result: Result<Column, _> = "5".parse();
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_column_addition_overflow() {
        let col1 = Column(u32::MAX);
        let col2 = Column(1);
        let _ = col1 + col2; // This should panic
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_column_subtraction_underflow() {
        let col1 = Column(0);
        let col2 = Column(1);
        let _ = col1 - col2; // This should panic
    }

    // Testes exclusivos de Column
    #[test]
    fn test_parse_col_valid() {
        assert_eq!(parse_col("A").unwrap(), Column(1));
        assert_eq!(parse_col("a").unwrap(), Column(1));
        assert_eq!(parse_col("Z").unwrap(), Column(26));
        assert_eq!(parse_col("z").unwrap(), Column(26));
        assert_eq!(parse_col("AA").unwrap(), Column(27));
        assert_eq!(parse_col("AB").unwrap(), Column(28));
        assert_eq!(parse_col("Zz").unwrap(), Column(26 * 26 + 26));
        assert_eq!(
            parse_col("zZz").unwrap(),
            Column(26 * 26 * 26 + 26 * 26 + 26)
        );
    }

    #[test]
    fn test_parse_col_invalid() {
        assert!(parse_col("A1").is_err());
        assert!(parse_col("1").is_err());
        assert!(parse_col("$").is_err());
    }

    #[test]
    fn test_number_to_letters() {
        assert_eq!(number_to_letters(1), "A");
        assert_eq!(number_to_letters(26), "Z");
        assert_eq!(number_to_letters(27), "AA");
        assert_eq!(number_to_letters(28), "AB");
        assert_eq!(number_to_letters(52), "AZ");
        assert_eq!(number_to_letters(53), "BA");
    }

    #[test]
    fn test_column_to_string_letters() {
        let col = Column(28);
        let s: String = col.into();
        assert_eq!(s, "AB");
    }

    #[test]
    fn test_column_from_str_letters() {
        let col: Column = parse_col("AB").unwrap();
        assert_eq!(col, Column(28));
    }

    #[test]
    fn test_column_from_str_non_alphabetic() {
        let col = parse_col("A1");
        assert!(matches!(col, Err(ColumnParseError::NonAlphabeticCharacter)));
    }
}
