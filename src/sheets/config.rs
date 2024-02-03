#[derive(serde::Deserialize, Debug, Clone)]
pub struct SpreadsheetConfig {
    pub priv_key: Box<str>,
    pub spreadsheet_id: Box<str>,
}
