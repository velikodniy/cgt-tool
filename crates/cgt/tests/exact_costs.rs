use cgt::{Config, calculate};
use rust_decimal_macros::dec;

#[test]
fn full_capital_return_equals_exact_purchase_cost() -> Result<(), Box<dyn std::error::Error>> {
    let transactions =
        cgt::dsl::parse(include_str!("../../../tests/inputs/ExactCapitalReturn.cgt"))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(3));
    assert_eq!(report.holdings[0].total_cost, dec!(0));
    Ok(())
}

#[test]
fn same_day_aggregation_preserves_the_half_penny_boundary() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = cgt::dsl::parse(include_str!("../../../tests/inputs/ExactMergedCost.cgt"))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(
        report.tax_years[0].disposals[0]
            .total_allowable_cost()
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero),
        dec!(7.01)
    );
    assert_eq!(report.holdings[0].quantity, dec!(0));
    assert_eq!(report.holdings[0].total_cost, dec!(0));
    Ok(())
}

#[test]
fn exact_partial_cost_does_not_fall_below_half_a_penny() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 7 @ 1.005\n\
         2024-03-01 SELL ABC 1 @ 2\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(
        report.tax_years[0].disposals[0].legs[0]
            .allowable_cost
            .round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero),
        dec!(1.01)
    );
    assert_eq!(report.holdings[0].total_cost, dec!(6.03));
    Ok(())
}
