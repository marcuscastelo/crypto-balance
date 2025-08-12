use std::{fmt::Formatter, num::ParseIntError, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row {
    index: u32,
}

impl Row {
    pub fn from_index(index: u32) -> Self {
        Row { index }
    }

    pub fn from_row(row: u32) -> Self {
        Row {
            index: row.saturating_sub(1), // Convert to zero-based index
        }
    }

    pub fn from_row_str(row: &str) -> Result<Self, ParseIntError> {
        let row = row.parse::<u32>()?;
        Ok(Row::from_row(row))
    }

    /// Returns the row number as a 1-based index.
    /// This is useful for representing rows in spreadsheets where rows are typically indexed starting from 1.
    /// # Examples
    /// ```
    /// use crypto_balance_core::domain::sheets::row::Row;
    /// let row = Row::from_index(0);
    /// assert_eq!(row.row(), "1");
    /// let row = Row::from_index(4);
    /// assert_eq!(row.row(), "5");
    /// let row = Row::from_index(25);
    /// assert_eq!(row.row(), "26");
    /// ```
    pub fn row(&self) -> String {
        self.index.saturating_add(1).to_string()
    }

    /// Returns the row index as a zero-based index.
    /// This is useful for internal calculations where rows are indexed starting from 0.
    /// # Examples
    /// ```
    /// use crypto_balance_core::domain::sheets::row::Row;
    /// let initial_row = Row::from_index(1);
    /// assert_eq!(initial_row.index(), 1);
    /// let delta_row = Row::from_index(4);
    /// assert_eq!(delta_row.index(), 4);
    /// let final_row = initial_row + delta_row;
    /// assert_eq!(final_row.index(), 5);
    /// ```
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl std::ops::Add for Row {
    type Output = Row;

    fn add(self, rhs: Row) -> Self::Output {
        Row::from_index(self.index.saturating_add(rhs.index))
    }
}

impl std::ops::Sub for Row {
    type Output = Row;

    fn sub(self, rhs: Row) -> Self::Output {
        Row::from_index(self.index.saturating_sub(rhs.index))
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.row())
    }
}

impl std::fmt::Debug for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Row(index: {}, row: {})", self.index(), self.row())
    }
}

impl FromStr for Row {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Row::from_row_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_addition() {
        let row1 = Row::from_index(1);
        let row2 = Row::from_index(3);
        assert_eq!(row1 + row2, Row::from_index(4));
    }

    #[test]
    fn test_row_subtraction() {
        let row1 = Row::from_index(5);
        let row2 = Row::from_index(3);
        assert_eq!(row1 - row2, Row::from_index(2));
    }

    #[test]
    fn test_row_complex_addition() {
        let row1 = Row::from_row(22);
        let row2 = Row::from_index(4);
        assert_eq!(row1 + row2, Row::from_row(26));
    }

    #[test]
    fn test_row_complex_subtraction() {
        let row1 = Row::from_row(26);
        let row2 = Row::from_index(4);
        assert_eq!(row1 - row2, Row::from_row(22));
    }

    #[test]
    fn test_row_complex_subtraction_saturate() {
        let row1 = Row::from_row(22);
        let row2 = Row::from_index(26);
        assert_eq!(row1 - row2, Row::from_index(0)); // Should saturate to zero
        assert_eq!(row1 - row2, Row::from_row(1)); // Should saturate to 1
        assert_eq!(row1 - row2, Row::from_row_str("1").unwrap()); // Should saturate to 1
    }

    #[test]
    fn test_row_display() {
        let row = Row::from_index(0);
        assert_eq!(row.to_string(), "1");
    }

    #[test]
    fn test_row_debug() {
        let row = Row::from_index(4);
        assert_eq!(format!("{:?}", row), "Row(index: 4, row: 5)");
    }

    #[test]
    fn test_index_from_row() {
        let row = Row::from_row(5);
        assert_eq!(row.index(), 4); // Zero-based index
    }

    #[test]
    fn test_row_from_index() {
        let row = Row::from_index(4);
        assert_eq!(row, Row::from_row(5));
    }

    #[test]
    fn test_getters_index_and_row() {
        let row = Row::from_index(3);
        assert_eq!(row.index(), 3);
        assert_eq!(row.row(), "4"); // 1-based index
    }

    #[test]
    fn test_row_from_str() {
        let row: Row = "5".parse().unwrap();
        assert_eq!(row, Row::from_row(5));
    }

    #[test]
    fn test_row_from_str_error() {
        let result: Result<Row, _> = "abc".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_index() {
        let row = Row::from_index(0);
        assert_eq!(row.index(), 0);
        assert_eq!(row.row(), "1"); // 1-based index
    }

    #[test]
    fn test_zero_row() {
        let row = Row::from_row(0);
        assert_eq!(row.index(), 0);
        assert_eq!(row.row(), "1"); // 1-based index
    }
}
