use cgt::model::is_valid_ticker;

#[test]
fn ticker_policy_matches_the_parser() {
    for symbol in [
        "FOO", "BRK/B", "BRK.B", "FOO-B", "FOO_B", "", ".FOO", "FOO B", "FOO#", "éFOO",
    ] {
        let parsed = cgt::dsl::parse(&format!("2024-01-01 BUY {symbol} 1 @ 10"));
        assert_eq!(parsed.is_ok(), is_valid_ticker(symbol), "{symbol}");
    }
}
