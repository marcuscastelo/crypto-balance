pub fn format_balance(balance: &str) -> anyhow::Result<f64> {
    // Get all text between $ and \n
    let (_, balance) = balance.split_once("$").unwrap_or(("", balance));
    let (balance, _) = balance.split_once("\n").unwrap_or((balance, ""));

    balance
        .replace(",", "")
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!(format!("Failed to parse balance: {:?}", balance)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_balance_decoration() {
        assert_eq!(format_balance("$1,234.56\n").unwrap(), 1234.56);
    }

    #[test]
    fn test_format_balance_no_decoration() {
        assert_eq!(format_balance("1234.56").unwrap(), 1234.56);
    }

    #[test]
    fn test_format_balance_no_decimal() {
        assert_eq!(format_balance("$1234\n").unwrap(), 1234.0);
    }
}
