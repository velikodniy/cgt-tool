use cgt::{Config, calculate, dsl};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn disposal_rounding_preserves_exact_legs_and_reconciles_displayed_totals()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = dsl::parse(
        "2024-01-01 BUY ABC 3 @ 0.015\n\
         2024-06-01 SELL ABC 3 @ 0.02\n\
         2024-06-02 BUY ABC 1 @ 0.015\n\
         2024-06-03 BUY ABC 1 @ 0.015\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let year = report.tax_years.first().ok_or("missing year")?;
    let disposal = year.disposals.first().ok_or("missing disposal")?;
    assert_eq!(year.total_gain, dec!(0.02));
    assert_eq!(year.total_allowable_cost, dec!(0.05));
    assert_eq!(disposal.total_gain(), dec!(0.02));
    assert_eq!(disposal.total_allowable_cost(), dec!(0.05));
    assert_eq!(disposal.legs[0].allowable_cost, dec!(0.015));
    assert_eq!(disposal.legs[0].gain_or_loss, dec!(0.005));

    let json = serde_json::to_value(&report)?;
    let output = &json["tax_years"][0]["disposals"][0];
    let matches = output["matches"].as_array().ok_or("missing matches")?;
    let money = |value: &serde_json::Value| -> Result<Decimal, Box<dyn std::error::Error>> {
        Ok(value.as_str().ok_or("missing money")?.parse()?)
    };
    let mut cost = Decimal::ZERO;
    let mut gain = Decimal::ZERO;
    for leg in matches {
        cost += money(&leg["allowable_cost"])?;
        gain += money(&leg["gain_or_loss"])?;
    }
    assert_eq!(cost, dec!(0.05));
    assert_eq!(gain, dec!(0.02));
    assert_eq!(money(&output["rounding_adjustment"])?, dec!(0.01));
    assert_eq!(
        money(&output["proceeds"])? - cost + money(&output["rounding_adjustment"])?,
        gain
    );
    assert!(
        cgt::plain::render(&cgt::report_display::prepare(&report)?)
            .contains("Rounding adjustment: £0.01")
    );
    Ok(())
}

#[test]
fn opposing_fractional_leg_gains_cancel_before_rounding() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = dsl::parse(
        "2024-01-01 BUY ABC 2 @ 1\n\
         2024-06-01 SELL ABC 2 @ 0.02\n\
         2024-06-02 BUY ABC 1 @ 0.015\n\
         2024-06-03 BUY ABC 1 @ 0.025\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let year = report.tax_years.first().ok_or("missing year")?;
    assert_eq!(year.total_gain, Decimal::ZERO);
    assert_eq!(year.total_loss, Decimal::ZERO);
    Ok(())
}
