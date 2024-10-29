use std::process::Command;

use crate::config::app_config::CONFIG;
pub async fn sonar() -> String {
    let output = Command::new("sh")
        .arg("./src/script/sonar.sh")
        .arg(CONFIG.blockchain.airdrops.solana.address.as_ref())
        .arg(CONFIG.scraping.sonar_watch.auth_token.as_ref())
        .arg(CONFIG.scraping.sonar_watch.turnstile_token.as_ref())
        .output()
        .expect("sonar.sh failed to start");

    String::from_utf8_lossy(&output.stdout).into_owned()
}
