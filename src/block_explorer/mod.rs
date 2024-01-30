pub mod etherscan;
pub mod prelude;

pub trait BlockExplorer {
    fn fetch_balance(&self, evm_address: &str) -> f64;
}
