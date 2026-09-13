use cgt::{Config, Transaction, calculate, dsl};
use rust_decimal::Decimal;

#[test]
fn zero_accumulation_preserves_cost_and_json_round_trip() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = dsl::parse(
        "2024-01-01 BUY FUND 10 @ 10\n\
         2024-06-01 ACCUMULATION FUND 0 TOTAL 20 TAX 3\n\
         2024-09-01 SELL FUND 1 @ 20\n",
    )?;
    let json = serde_json::to_string(&transactions)?;
    let reparsed: Vec<Transaction> = serde_json::from_str(&json)?;
    assert_eq!(transactions, reparsed);

    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let year = report.tax_years.first().ok_or("missing tax year")?;
    assert_eq!(year.total_allowable_cost, Decimal::from(10));
    assert_eq!(year.net_gain, Decimal::from(10));
    assert_eq!(year.dividend_income, Decimal::ZERO);
    assert_eq!(year.dividend_tax_paid, Decimal::ZERO);
    let holding = report.holdings.first().ok_or("missing holding")?;
    assert_eq!(holding.quantity, Decimal::from(9));
    assert_eq!(holding.total_cost, Decimal::from(90));
    Ok(())
}

#[test]
fn zero_accumulation_needs_no_holding_or_fx_rate() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = dsl::parse("2099-01-01 ACCUMULATION FUND 0 TOTAL 20 USD TAX 3 USD\n")?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert!(report.tax_years.is_empty());
    assert!(report.holdings.is_empty());
    Ok(())
}
