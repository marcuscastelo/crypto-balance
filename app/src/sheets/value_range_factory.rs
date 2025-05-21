use google_sheets4::api::ValueRange;
use serde_json::Value;

pub trait ValueRangeFactory {
    fn from_rows<T: AsRef<str>>(rows: &[T]) -> Self;
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
}
