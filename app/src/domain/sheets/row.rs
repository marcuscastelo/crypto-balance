use std::{fmt::Formatter, num::ParseIntError, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row(u32);

impl Row {
    pub fn new(value: u32) -> Self {
        if value == 0 {
            panic!("Row number cannot be zero");
        }
        Row(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::ops::Add for Row {
    type Output = Row;

    fn add(self, rhs: Row) -> Self::Output {
        Row(self
            .0
            .checked_add(rhs.0)
            .expect("attempt to add with overflow"))
    }
}

impl std::ops::Sub for Row {
    type Output = Row;

    fn sub(self, rhs: Row) -> Self::Output {
        Row(self
            .0
            .checked_sub(rhs.0)
            .expect("attempt to subtract with overflow"))
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Conversions: Others -> Row

impl From<u32> for Row {
    fn from(value: u32) -> Self {
        Row(value)
    }
}

impl FromStr for Row {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Row(s.parse()?))
    }
}

/// Conversions: Row -> u32

impl From<Row> for u32 {
    fn from(row: Row) -> Self {
        row.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_addition() {
        let row1 = Row(5);
        let row2 = Row(3);
        assert_eq!(row1 + row2, Row(8));
    }

    #[test]
    fn test_row_subtraction() {
        let row1 = Row(5);
        let row2 = Row(3);
        assert_eq!(row1 - row2, Row(2));
    }

    #[test]
    fn test_row_display() {
        let row = Row(5);
        assert_eq!(row.to_string(), "5");
    }

    #[test]
    fn test_row_from_u32() {
        let row: Row = 5.into();
        assert_eq!(row, Row(5));
    }

    #[test]
    fn test_row_from_str() {
        let row: Row = "5".parse().unwrap();
        assert_eq!(row, Row(5));
    }

    #[test]
    fn test_row_to_u32() {
        let row = Row(5);
        let value: u32 = row.into();
        assert_eq!(value, 5);
    }

    #[test]
    fn test_row_from_str_error() {
        let result: Result<Row, _> = "abc".parse();
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_row_addition_overflow() {
        let row1 = Row(u32::MAX);
        let row2 = Row(1);
        let _ = row1 + row2; // This should panic
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_row_subtraction_underflow() {
        let row1 = Row(0);
        let row2 = Row(1);
        let _ = row1 - row2; // This should panic
    }
}
