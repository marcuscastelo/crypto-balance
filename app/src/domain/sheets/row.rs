use std::{fmt::Formatter, num::ParseIntError, str::FromStr};

// TODO: Forbid 0 or change to 0-indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row(pub u32);

impl std::ops::Add for Row {
    type Output = Row;

    fn add(self, rhs: Row) -> Self::Output {
        Row(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Row {
    type Output = Row;

    fn sub(self, rhs: Row) -> Self::Output {
        Row(self.0 - rhs.0)
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
