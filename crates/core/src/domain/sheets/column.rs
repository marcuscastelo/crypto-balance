use core::panic;
use std::{fmt::Formatter, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Column {
    index: u32,
}

impl Column {
    pub fn from_index(index: u32) -> Self {
        Column { index }
    }

    pub fn from_col(col: u32) -> Self {
        Column {
            index: col.saturating_sub(1), // Convert to zero-based index
        }
    }

    pub fn from_col_str(col: &str) -> Result<Self, ColumnParseError> {
        let col = convert_letters_to_column_number(col)?;
        Ok(Column::from_col(col))
    }

    /// Returns the column number as a 1-based index.
    /// This is useful for representing columns in spreadsheets where columns are typically indexed starting from 1.
    ///
    /// # Examples
    /// ```
    /// use domain::sheets::column::Column;
    /// let col = Column::from_index(0);
    /// assert_eq!(col.column(), "A");
    /// let col = Column::from_index(25);
    /// assert_eq!(col.column(), "Z");
    /// let col = Column::from_index(26);
    /// assert_eq!(col.column(), "AA");
    /// ```
    pub fn column(&self) -> String {
        convert_number_to_column_letters(self.column_num())
    }

    /// Returns the column number as a 1-based index.
    /// # Examples
    /// ```
    /// use domain::sheets::column::Column;
    /// let initial_col = Column::from_index(0);
    /// assert_eq!(initial_col.column_num(), 1);
    /// let delta_col = Column::from_index(3);
    /// assert_eq!(delta_col.column_num(), 4);
    /// let final_col = initial_col + delta_col;
    /// assert_eq!(final_col.column_num(), 5);
    /// ```
    pub fn column_num(&self) -> u32 {
        self.index.saturating_add(1)
    }

    /// Returns the column index as a zero-based index.
    /// This is useful for internal calculations where columns are indexed starting from 0.
    /// # Examples
    /// ```
    /// use domain::sheets::column::Column;
    /// let initial_col = Column::from_index(1);
    /// assert_eq!(initial_col.index(), 1);
    /// let delta_col = Column::from_index(4);
    /// assert_eq!(delta_col.index(), 4);
    /// let final_col = initial_col + delta_col;
    /// assert_eq!(final_col.index(), 5);
    /// ```
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl std::ops::Add for Column {
    type Output = Column;

    fn add(self, rhs: Column) -> Self::Output {
        Column::from_index(self.index.saturating_add(rhs.index))
    }
}

impl std::ops::Sub for Column {
    type Output = Column;

    fn sub(self, rhs: Column) -> Self::Output {
        Column::from_index(self.index.saturating_sub(rhs.index))
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.column())
    }
}

impl std::fmt::Debug for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Show both the numeric and letter representation
        write!(
            f,
            "Column(index: {}, column: {})",
            self.index(),
            self.column()
        )
    }
}

/// Conversions: Others -> Column

// impl From<u32> for Column {
//     fn from(value: u32) -> Self {
//         Column(value)
//     }
// }

#[derive(Debug, Error)]
pub enum ColumnParseError {
    #[error("Non-alphabetic character found in column string")]
    NonAlphabeticCharacter,
}

impl FromStr for Column {
    type Err = ColumnParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let column = convert_letters_to_column_number(s)?;
        Ok(Column::from_col(column))
    }
}

/// Conversions: Column -> Others

// impl From<Column> for u32 {
//     fn from(col: Column) -> Self {
//         col.0
//     }
// }

// impl From<Column> for String {
//     fn from(col: Column) -> Self {
//         number_to_letters(col.0)
//     }
// }

fn convert_letters_to_column_number<T: AsRef<str>>(col_str: T) -> Result<u32, ColumnParseError> {
    if col_str.as_ref().chars().any(|c| !c.is_ascii_alphabetic()) {
        return Err(ColumnParseError::NonAlphabeticCharacter);
    }

    let col_num_index_1 = col_str
        .as_ref()
        .chars()
        .map(|c| c.to_ascii_uppercase())
        .fold(0, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    Ok(col_num_index_1)
}

fn convert_number_to_column_letters(number: u32) -> String {
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
        let col1 = Column::from_index(5);
        let col2 = Column::from_index(3);
        assert_eq!(col1 + col2, Column::from_index(8));
    }

    #[test]
    fn test_column_subtraction() {
        let col1 = Column::from_index(5);
        let col2 = Column::from_index(3);
        assert_eq!(col1 - col2, Column::from_index(2));
    }

    #[test]
    fn test_column_complex_addition() {
        let col1 = Column::from_col(22);
        let col2 = Column::from_index(4);
        assert_eq!(col1 + col2, Column::from_col(26));
    }

    #[test]
    fn test_column_complex_subtraction() {
        let col1 = Column::from_col(26);
        let col2 = Column::from_index(4);
        assert_eq!(col1 - col2, Column::from_col(22));
    }

    #[test]
    fn test_column_complex_addition_str() {
        let col1 = Column::from_col_str("C").unwrap();
        let col2 = Column::from_index(2);
        assert_eq!(col1 + col2, Column::from_col_str("E").unwrap())
    }

    #[test]
    fn test_column_complex_subtraction_str() {
        let col1 = Column::from_col_str("C").unwrap();
        let col2 = Column::from_index(2);
        assert_eq!(col1 - col2, Column::from_col_str("A").unwrap())
    }

    #[test]
    fn test_column_complex_subtraction_str_saturate() {
        let col1 = Column::from_col_str("C").unwrap();
        let col2 = Column::from_index(3);
        assert_eq!(col1 - col2, Column::from_index(0)); // Should saturate to zero
        assert_eq!(col1 - col2, Column::from_col(1)); // Should saturate to 1
        assert_eq!(col1 - col2, Column::from_col_str("A").unwrap())
    }

    #[test]
    fn test_column_display_a() {
        let col = Column::from_col(1);
        assert_eq!(col.to_string(), "A");
    }

    #[test]
    fn test_column_display_z() {
        let col = Column::from_col(26);
        assert_eq!(col.to_string(), "Z");
    }

    #[test]
    fn test_column_display_aa() {
        let col = Column::from_col(26 * 1 + 1);
        assert_eq!(col.to_string(), "AA");
    }

    #[test]
    fn test_column_display_ab() {
        let col = Column::from_col(26 * 1 + 2);
        assert_eq!(col.to_string(), "AB");
    }

    #[test]
    fn test_column_display_az() {
        let col = Column::from_col(26 * 1 + 26);
        assert_eq!(col.to_string(), "AZ");
    }

    #[test]
    fn test_column_display_ba() {
        let col = Column::from_col(26 * 2 + 1);
        assert_eq!(col.to_string(), "BA");
    }

    #[test]
    fn test_column_display_za() {
        let col = Column::from_col(26 * 26 + 1);
        assert_eq!(col.to_string(), "ZA");
    }

    #[test]
    fn test_column_display_zza() {
        let col = Column::from_col(26 * 26 * 26 + 26 * 26 + 1);
        assert_eq!(col.to_string(), "ZZA");
    }

    #[test]
    fn test_column_display_zzy() {
        let col = Column::from_col(26 * 26 * 26 + 26 * 26 + 25);
        assert_eq!(col.to_string(), "ZZY");
    }

    #[test]
    fn test_column_from_str() {
        let col: Column = "C".parse().unwrap();
        assert_eq!(col, Column::from_col(3));
    }

    #[test]
    fn test_column_from_str_invalid() {
        let result: Result<Column, _> = "C1".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_column_from_str_lower() {
        let col: Column = "a".parse().unwrap();
        assert_eq!(col, Column::from_col(1));
    }

    #[test]
    fn test_column_from_str_upper() {
        let col: Column = "A".parse().unwrap();
        assert_eq!(col, Column::from_col(1));
    }

    #[test]
    fn test_column_from_str_error() {
        let result: Result<Column, _> = "5".parse();
        assert!(result.is_err());
    }

    // Testes exclusivos de Column
    #[test]
    fn test_parse_col_valid() {
        assert_eq!(
            convert_letters_to_column_number("A").unwrap(),
            Column::from_col(1).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("a").unwrap(),
            Column::from_col(1).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("Z").unwrap(),
            Column::from_col(26).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("z").unwrap(),
            Column::from_col(26).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("AA").unwrap(),
            Column::from_col(27).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("AB").unwrap(),
            Column::from_col(28).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("Zz").unwrap(),
            Column::from_col(26 * 26 + 26).column_num()
        );
        assert_eq!(
            convert_letters_to_column_number("zZz").unwrap(),
            Column::from_col(26 * 26 * 26 + 26 * 26 + 26).column_num()
        );
    }

    #[test]
    fn test_parse_col_invalid() {
        assert!(convert_letters_to_column_number("A1").is_err());
        assert!(convert_letters_to_column_number("1").is_err());
        assert!(convert_letters_to_column_number("$").is_err());
    }

    #[test]
    fn test_number_to_letters() {
        assert_eq!(convert_number_to_column_letters(1), "A");
        assert_eq!(convert_number_to_column_letters(26), "Z");
        assert_eq!(convert_number_to_column_letters(27), "AA");
        assert_eq!(convert_number_to_column_letters(28), "AB");
        assert_eq!(convert_number_to_column_letters(52), "AZ");
        assert_eq!(convert_number_to_column_letters(53), "BA");
    }

    #[test]
    fn test_column_to_string_letters() {
        let col = Column::from_col(28);
        let s: String = col.column();
        assert_eq!(s, "AB");
    }

    #[test]
    fn test_column_from_str_letters() {
        let col = convert_letters_to_column_number("AB").unwrap();
        assert_eq!(col, 28);
    }

    #[test]
    fn test_column_from_str_non_alphabetic() {
        let col = convert_letters_to_column_number("A1");
        assert!(matches!(col, Err(ColumnParseError::NonAlphabeticCharacter)));
    }
}
