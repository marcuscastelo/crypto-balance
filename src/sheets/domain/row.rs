use std::{fmt::Formatter, num::ParseIntError, ops::Deref, str::FromStr};

// TODO: Forbid 0 or change to 0-indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row(pub u32);

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

impl std::fmt::Display for Row {
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

impl From<Row> for u32 {
    fn from(row: Row) -> Self {
        row.0
    }
}
