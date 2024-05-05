// use chrono::Utc;
// use hmac::{Hmac, Mac};
// use sha2::Sha256;

// pub struct BybitClient {
//     api_key: Box<str>,
//     secret_key: Box<str>,
// }

// impl BybitClient {
//     pub fn new(api_key: Box<str>, secret_key: Box<str>) -> Self {
//         Self { api_key, secret_key }
//     }

//     pub async fn get_wallet_balance(self) {
//         // GET /v5/account/wallet-balance?accountType=UNIFIED HTTP/1.1
//         // Host: api-testnet.bybit.com
//         // X-BAPI-SIGN: XXXXX
//         // X-BAPI-API-KEY: XXXXX
//         // X-BAPI-TIMESTAMP: 1672125440406
//         // X-BAPI-RECV-WINDOW: 5000

//         let url = "https://api.bybit.com/v5/account/wallet-balance?accountType=UNIFIED";
//         let client = reqwest::Client::new();

//         let timestamp = Utc::now().timestamp_millis();

//         let response = client
//             .get(url)
//             .header("Host", "api.bybit.com")
//             .header("X-BAPI-API-KEY", self.api_key.as_ref())
//             .header("X-BAPI-SIGN", self.secret_key.as_ref())
//             .header("X-BAPI-TIMESTAMP", timestamp)
//             .header("X-BAPI-RECV-WINDOW", 5000)
//             .send()
//             .await.expect("Should send request");

//         println!("Response: {:?}", response);
//     }
// }