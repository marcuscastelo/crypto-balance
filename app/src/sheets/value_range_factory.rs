use google_sheets4::api::ValueRange;
use serde_json::Value;
use std::borrow::Cow;

pub trait ValueRangeFactory {
    fn from_single_cell<'a, T: Into<Cow<'a, str>> + Clone>(cell_value: T) -> Self;
    fn from_single_column<'a, T: Into<Cow<'a, str>> + Clone>(
        column_values: &[T],
        row_count: u32,
    ) -> Self;
    fn from_two_columns<'a, T: Into<Cow<'a, str>> + Clone>(
        column_values: &[T],
        column_values2: &[T],
        row_count: u32,
    ) -> Self;
}

fn wrap_value<'a, T: Into<Cow<'a, str>>>(value: T) -> Value {
    Value::String(value.into().into_owned())
}

impl ValueRangeFactory for ValueRange {
    fn from_single_cell<'a, T: Into<Cow<'a, str>> + Clone>(cell_value: T) -> Self {
        ValueRange {
            major_dimension: None,
            range: None,
            values: Some(vec![vec![wrap_value(cell_value)]]),
        }
    }

    fn from_single_column<'a, T: Into<Cow<'a, str>> + Clone>(
        column_values: &[T],
        row_count: u32,
    ) -> Self {
        let mut values = column_values
            .iter()
            .map(|col_item| vec![wrap_value(col_item.clone())])
            .collect::<Vec<_>>();

        values.extend(
            (column_values.len()..row_count as usize)
                .map(|_| vec![wrap_value("")])
                .collect::<Vec<_>>(),
        );

        Self {
            major_dimension: Some("ROWS".to_string()),
            range: None,
            values: Some(values),
        }
    }

    fn from_two_columns<'a, T: Into<Cow<'a, str>> + Clone>(
        column_values: &[T],
        column_values2: &[T],
        row_count: u32,
    ) -> Self {
        let mut values = column_values
            .iter()
            .zip(column_values2.iter())
            .map(|(col1_item, col2_item)| {
                vec![wrap_value(col1_item.clone()), wrap_value(col2_item.clone())]
            })
            .collect::<Vec<_>>();

        values.extend(
            (column_values.len()..row_count as usize)
                .map(|_| vec![wrap_value(""), wrap_value("")])
                .collect::<Vec<_>>(),
        );

        Self {
            major_dimension: Some("ROWS".to_string()),
            range: None,
            values: Some(values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test for wrap "1" -> Value::String("1")
    #[test]
    fn test_wrap_value() {
        let value = wrap_value("1");
        assert_eq!(value, Value::String("1".to_string()));
    }

    // Test for ValueRange::from_single_cell("1") -> ValueRange { values: Some(vec![vec![Value::String("1")]]) }
    #[test]
    fn test_from_single_cell() {
        let value_range = ValueRange::from_single_cell("1");
        assert_eq!(
            value_range.major_dimension, None,
            "Major dimension should be None"
        );
        assert_eq!(value_range.range, None, "Range should be None");
        assert_eq!(
            value_range.values,
            Some(vec![vec![Value::String("1".to_string())]]),
            "Values should be a single cell with Value::String(\"1\")"
        );
    }

    // Test for ValueRange::from_single_column(["1", "2"]) -> ValueRange { values: Some(vec![vec![Value::String("1")], vec![Value::String("2")]]) }
    #[test]
    fn test_from_single_column() {
        let value_range = ValueRange::from_single_column(&["1", "2"], 1);
        assert_eq!(
            value_range.major_dimension,
            Some("ROWS".to_string()),
            "Major dimension should be ROWS"
        );
        assert_eq!(value_range.range, None, "Range should be None");
        assert_eq!(
            value_range.values,
            Some(vec![
                vec![Value::String("1".to_string())],
                vec![Value::String("2".to_string())]
            ]),
            "Values should be a single column with Value::String(\"1\") and Value::String(\"2\")"
        );
    }

    // Test for ValueRange::from_two_columns(["1", "2"], ["3", "4"]) -> ValueRange { values: Some(vec![vec![Value::String("1"), Value::String("3")], vec![Value::String("2"), Value::String("4")]]) }
    #[test]
    fn test_from_two_columns() {
        let value_range = ValueRange::from_two_columns(&["1", "2"], &["3", "4"], 1);
        assert_eq!(
            value_range.major_dimension,
            Some("ROWS".to_string()),
            "Major dimension should be ROWS"
        );
        assert_eq!(value_range.range, None, "Range should be None");
        assert_eq!(
            value_range.values,
            Some(vec![
                vec![Value::String("1".to_string()), Value::String("3".to_string())],
                vec![Value::String("2".to_string()), Value::String("4".to_string())]
            ]),
            "Values should be two columns with Value::String(\"1\") and Value::String(\"3\") and Value::String(\"2\") and Value::String(\"4\")"
        );
    }
}
