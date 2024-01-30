const ETH_IN_WEI: u64 = 1000000000000000000;

mod app_config;

use app_config::CONFIG;

fn main() {
    fetch_balance(&CONFIG.evm_address);
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct FetchBalanceResponse {
    status: String,
    message: String,
    result: String,
}

fn fetch_balance(evm_address: &str) {
    let api_key = &CONFIG.api_key;
    let url = format!(
        "https://api.etherscan.io/api\
            ?module=account\
            &action=balance\
            &address={evm_address}\
            &tag=latest\
            &apikey={api_key}"
    );
    let resp = reqwest::blocking::get(url).unwrap().text().unwrap();
    let resp: FetchBalanceResponse = serde_json::from_str(&resp).unwrap();
    let balance = resp.result.parse::<f64>().unwrap() / (ETH_IN_WEI as f64);
    println!("Balance of {} is {} ETH", evm_address, balance);
}
