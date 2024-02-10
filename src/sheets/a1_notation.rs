use google_sheets4::api::GridRange;

pub trait A1Notation {
    fn to_a1_notation(&self, sheet_name: &str) -> String;
    fn from_a1_notation(a1_notation: &str, sheet_id: i32) -> Self;
}

pub fn number_to_letter(number: u32) -> String {
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

impl A1Notation for GridRange {
    fn to_a1_notation(&self, sheet_name: &str) -> String {
        let start_row = (self.start_row_index.unwrap() + 1) as u32;
        let start_col = (self.start_column_index.unwrap() + 1) as u32;
        let end_row = self.end_row_index.unwrap() as u32;
        let end_col = self.end_column_index.unwrap() as u32;
        let start_col_letter = number_to_letter(start_col);
        let end_col_letter = number_to_letter(end_col);

        format!(
            "'{}'!{}{}:{}{}",
            sheet_name, start_col_letter, start_row, end_col_letter, end_row
        )
    }

    fn from_a1_notation(a1_notation: &str, sheet_id: i32) -> Self {
        let mut parts = a1_notation.split(':');
        let start = parts.next().unwrap();
        let end = parts.next().unwrap();
        let mut start_parts = start.chars();
        let mut end_parts = end.chars();
        let start_col_letter = start_parts.next().unwrap();
        let start_row = start_parts.collect::<String>().parse::<i32>().unwrap();
        let end_col_letter = end_parts.next().unwrap();
        let end_row = end_parts.collect::<String>().parse::<i32>().unwrap();
        let start_col = start_col_letter as i32 - 64;
        let end_col = end_col_letter as i32 - 64;

        GridRange {
            sheet_id: Some(sheet_id),
            start_row_index: Some(start_row - 1),
            start_column_index: Some(start_col - 1),
            end_row_index: Some(end_row - 1),
            end_column_index: Some(end_col - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_letter() {
        assert_eq!(number_to_letter(1), "A");
        assert_eq!(number_to_letter(26), "Z");
        assert_eq!(number_to_letter(27), "AA");
        assert_eq!(number_to_letter(52), "AZ");
        assert_eq!(number_to_letter(53), "BA");
        assert_eq!(number_to_letter(702), "ZZ");
        assert_eq!(number_to_letter(703), "AAA");
    }
}
