#[derive(serde::Deserialize, Debug, Clone)]
pub struct SpreadsheetConfig {
    pub priv_key: Box<str>, // https://console.cloud.google.com/iam-admin/serviceaccounts/details/106085307439944090164;edit=true/keys?project=cryptosheets-355223
    pub spreadsheet_id: Box<str>,
}
