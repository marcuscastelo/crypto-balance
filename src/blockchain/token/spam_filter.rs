pub fn check_spam(token_symbol: &str, token_name: &str) -> bool {
    let has_visit = |s: &str| s.to_uppercase().contains("VISIT");
    let has_access =
        |s: &str| s.to_uppercase().contains("ACCES") || s.to_uppercase().contains("ACESS");

    let has_www = |s: &str| s.to_uppercase().contains("WWW");
    let has_com = |s: &str| s.to_uppercase().contains(".COM");
    let has_co = |s: &str| s.to_uppercase().contains(".CO");
    let has_net = |s: &str| s.to_uppercase().contains(".NET");
    let has_io = |s: &str| s.to_uppercase().contains(".IO");
    let has_eligible = |s: &str| s.to_uppercase().contains("ELIGIBLE");
    let has_airdrop = |s: &str| s.to_uppercase().contains("AIRDROP");
    let has_claim =
        |s: &str| s.to_uppercase().contains("CLAIM") || s.to_uppercase().contains("СLАLМ");
    let has_free = |s: &str| s.to_uppercase().contains("FREE");
    let has_voucher = |s: &str| s.to_uppercase().contains("VOUCHER");
    let has_non_ascii = |s: &str| s.chars().any(|c| !c.is_ascii());

    let has_spam = |s: &str| {
        has_visit(s)
            || has_access(s)
            || has_www(s)
            || has_com(s)
            || has_co(s)
            || has_net(s)
            || has_io(s)
            || has_eligible(s)
            || has_airdrop(s)
            || has_claim(s)
            || has_free(s)
            || has_voucher(s)
            || has_non_ascii(s)
    };

    has_spam(token_symbol) || has_spam(token_name)
}
