use google_sheets4::api::ValueRange;
use serde_json::Value;

pub trait ValueRangeFactory {
    fn from_rows<T: AsRef<str>>(rows: &[T]) -> Self;
    fn from_cols<T: AsRef<str>>(rows: &[T]) -> Self;
    fn from_str<T: AsRef<str>>(s: T) -> Self;
}

fn vec_string_to_values<T: AsRef<str>>(rows: &[T]) -> Vec<Vec<Value>> {
    rows.iter()
        .map(|row| vec![Value::String(row.as_ref().to_owned())])
        .collect()
}

impl ValueRangeFactory for ValueRange {
    fn from_str<T: AsRef<str>>(s: T) -> Self {
        ValueRange {
            major_dimension: None,
            range: None,
            values: Some(vec![vec![Value::String(s.as_ref().to_owned())]]),
        }
    }

    fn from_rows<T: AsRef<str>>(rows: &[T]) -> Self {
        let values = vec_string_to_values(rows);
        Self {
            major_dimension: Some("ROWS".to_string()),
            range: None,
            values: Some(values),
        }
    }

    fn from_cols<T: AsRef<str>>(rows: &[T]) -> Self {
        let values = vec_string_to_values(rows);
        Self {
            major_dimension: Some("COLUMNS".to_string()),
            range: None,
            values: Some(values),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_string_to_values() {
        let rows = vec!["a", "b", "c"];
        let values = vec_string_to_values(&rows);
        assert_eq!(
            values,
            vec![
                vec![Value::String("a".to_string())],
                vec![Value::String("b".to_string())],
                vec![Value::String("c".to_string())],
            ]
        );
    }

    #[test]
    fn test_from_rows() {
        let rows = vec!["a", "b", "c"];
        let value_range = ValueRange::from_rows(&rows);
        assert_eq!(value_range.major_dimension, Some("ROWS".to_string()));
        assert_eq!(value_range.values, Some(vec_string_to_values(&rows)));
    }

    #[test]
    fn test_from_cols() {
        let rows = vec!["a", "b", "c"];
        let value_range = ValueRange::from_cols(&rows);
        assert_eq!(value_range.major_dimension, Some("COLUMNS".to_string()));
        assert_eq!(value_range.values, Some(vec_string_to_values(&rows)));
    }
}
