// pub async fn read(
//     hub: &Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
//     config: &Config,
// ) -> Result<(hyper::Response<hyper::Body>, ValueRange), Error> {
//     hub.spreadsheets()
//         .values_get(&config.sheet_id, &config.deposit_range_input)
//         .doit()
//         .await
// }
