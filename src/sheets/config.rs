#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub priv_key: String,
    pub sheet_id: String,
    pub deposit_range_input: String,
    pub deposit_range_output: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            priv_key: String::from("***REMOVED***"),
            sheet_id: String::from("***REMOVED***"),
            deposit_range_input: String::from("Test!A1:A25"),
            deposit_range_output: String::from("Test!B1"),
        }
    }
}
