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
///
/// Schwab exports do not include share quantity for dividend rows, so quantity is emitted
/// as `1` and the cash value is emitted via `TOTAL` to produce valid CGT DSL.
pub fn format_dividend(
    date: &NaiveDate,
    symbol: &str,
    total_value: Decimal,
    currency: &str,
    tax: Option<Decimal>,
) -> String {
    let mut line = format!(
        "{} DIVIDEND {} 1 TOTAL {} {}",
        format_date(date),
        symbol,
        format_amount(total_value),
        currency
    );

    if let Some(tax_amount) = tax
        && tax_amount > Decimal::ZERO
    {
        line.push_str(&format!(" TAX {} {}", format_amount(tax_amount), currency));
    }

    line
}

/// Format a comment line
pub fn format_comment(text: &str) -> String {
    format!("# {}", text)
}

/// Generate header comments for a converted file
pub fn generate_header(broker: &str, source_files: &[String], skipped_count: usize) -> String {
    let mut lines = vec![
        format_comment(&format!("Converted from {} export", broker)),
        format_comment(&format!("Source files: {}", source_files.join(", "))),
        format_comment(&format!("Converted: {}", chrono::Utc::now().to_rfc3339())),
    ];

    if skipped_count > 0 {
        lines.push(format_comment(&format!(
            "SKIPPED: {} transactions not CGT-relevant",
            skipped_count
        )));
    }

    lines.push(String::new()); // Blank line after header
    lines.join("\n")
}
