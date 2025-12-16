use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Format a date as YYYY-MM-DD
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Format a decimal amount with appropriate precision
pub fn format_amount(amount: Decimal) -> String {
    amount.to_string()
}

/// Format a BUY transaction line
pub fn format_buy(
    date: &NaiveDate,
    symbol: &str,
    quantity: Decimal,
    price: Decimal,
    currency: &str,
    expenses: Option<Decimal>,
) -> String {
    let mut line = format!(
        "{} BUY {} {} @ {} {}",
        format_date(date),
        symbol,
        format_amount(quantity),
        format_amount(price),
        currency
    );

    if let Some(exp) = expenses
        && exp > Decimal::ZERO
    {
        line.push_str(&format!(" FEES {} {}", format_amount(exp), currency));
    }

    line
}

/// Format a SELL transaction line
pub fn format_sell(
    date: &NaiveDate,
    symbol: &str,
    quantity: Decimal,
    price: Decimal,
    currency: &str,
    expenses: Option<Decimal>,
) -> String {
    let mut line = format!(
        "{} SELL {} {} @ {} {}",
        format_date(date),
        symbol,
        format_amount(quantity),
        format_amount(price),
        currency
    );

    if let Some(exp) = expenses
        && exp > Decimal::ZERO
    {
        line.push_str(&format!(" FEES {} {}", format_amount(exp), currency));
    }

    line
}

/// Format a DIVIDEND transaction line
pub fn format_dividend(
    date: &NaiveDate,
    symbol: &str,
    amount: Decimal,
    currency: &str,
    tax: Option<Decimal>,
) -> String {
    let mut line = format!(
        "{} DIVIDEND {} {} {}",
        format_date(date),
        symbol,
        format_amount(amount),
        currency
    );

    if let Some(tax_amount) = tax
        && tax_amount > Decimal::ZERO
    {
        line.push_str(&format!(" TAX {} {}", format_amount(tax_amount), currency));
    }

    line
}

/// Format a SPLIT transaction line
pub fn format_split(date: &NaiveDate, symbol: &str, ratio: &str) -> String {
    format!("{} SPLIT {} {}", format_date(date), symbol, ratio)
}

/// Format a comment line
pub fn format_comment(text: &str) -> String {
    format!("# {}", text)
}

/// Generate header comments for a converted file
pub fn generate_header(broker: &str, source_files: &[String], warnings: &[String]) -> String {
    let mut lines = vec![
        format_comment(&format!("Converted from {} export", broker)),
        format_comment(&format!("Source files: {}", source_files.join(", "))),
        format_comment(&format!("Converted: {}", chrono::Utc::now().to_rfc3339())),
    ];

    if !warnings.is_empty() {
        lines.push(format_comment(&format!(
            "WARNING: {} transactions skipped (not CGT-relevant)",
            warnings.len()
        )));
    }

    lines.push(String::new()); // Blank line after header
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_format_buy_without_expenses() {
        let date = NaiveDate::from_ymd_opt(2023, 4, 25).unwrap();
        let result = format_buy(&date, "XYZZ", dec!(67.2), dec!(125.6445), "USD", None);
        assert_eq!(result, "2023-04-25 BUY XYZZ 67.2 @ 125.6445 USD");
    }

    #[test]
    fn test_format_buy_with_expenses() {
        let date = NaiveDate::from_ymd_opt(2023, 5, 10).unwrap();
        let result = format_buy(
            &date,
            "XYZZ",
            dec!(10),
            dec!(130.00),
            "USD",
            Some(dec!(4.95)),
        );
        assert_eq!(result, "2023-05-10 BUY XYZZ 10 @ 130.00 USD FEES 4.95 USD");
    }

    #[test]
    fn test_format_sell() {
        let date = NaiveDate::from_ymd_opt(2023, 6, 14).unwrap();
        let result = format_sell(
            &date,
            "XYZZ",
            dec!(62.601495),
            dec!(113.75),
            "USD",
            Some(dec!(0.17)),
        );
        assert_eq!(
            result,
            "2023-06-14 SELL XYZZ 62.601495 @ 113.75 USD FEES 0.17 USD"
        );
    }

    #[test]
    fn test_format_dividend_with_tax() {
        let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();
        let result = format_dividend(&date, "FOO", dec!(50.00), "USD", Some(dec!(7.50)));
        assert_eq!(result, "2023-07-15 DIVIDEND FOO 50.00 USD TAX 7.50 USD");
    }

    #[test]
    fn test_format_comment() {
        assert_eq!(format_comment("Test comment"), "# Test comment");
    }
}
