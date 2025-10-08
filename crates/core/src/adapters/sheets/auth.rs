use google_sheets4::oauth2::{self, authenticator::Authenticator};
use google_sheets4::{hyper, hyper_rustls};

use crate::adapters::config::sheets_config::SpreadsheetConfig;

pub async fn auth(
    config: &SpreadsheetConfig,
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) -> Authenticator<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    let priv_key_path = config.priv_key.as_ref();
    let secret: oauth2::ServiceAccountKey = oauth2::read_service_account_key(priv_key_path)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Google Sheets update failed: could not read service account private key at '{}'. Reason: {}. Please provide a valid service account private key to enable Google Sheets integration.",
                priv_key_path, err
            )
        });

    oauth2::ServiceAccountAuthenticator::with_client(secret, client.clone())
        .build()
        .await
        .expect("could not create an authenticator")
}
