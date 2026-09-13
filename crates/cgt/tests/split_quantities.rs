use cgt::{Config, MatchRule, calculate};
use rust_decimal_macros::dec;

#[test]
fn split_thirds_preserve_buy_and_pool_quantities() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(include_str!("../../../tests/inputs/SplitThirds.cgt"))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(26));
    assert_eq!(report.tax_years[0].disposals[0].quantity, dec!(2));
    assert_eq!(
        report.tax_years[0].disposals[0].legs[0].allowable_cost,
        dec!(8)
    );
    Ok(())
}

#[test]
fn inverse_thirds_leave_whole_shares_for_a_later_consolidation()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(include_str!("../../../tests/inputs/UnsplitThirds.cgt"))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(2));
    assert_eq!(report.holdings[0].total_cost, dec!(120));
    Ok(())
}

#[test]
fn split_thirds_preserve_the_entire_same_day_reservation() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 10 @ 10\n\
         2024-02-01 SELL ABC 2 @ 12\n\
         2024-02-02 SPLIT ABC RATIO 3\n\
         2024-02-20 BUY ABC 3 @ 4\n\
         2024-02-20 SELL ABC 1 @ 5\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let later = &report.tax_years[0].disposals[1];
    assert_eq!(later.legs.len(), 1);
    assert_eq!(later.legs[0].rule, MatchRule::SameDay);
    assert_eq!(later.legs[0].quantity, dec!(1));
    assert_eq!(report.holdings[0].quantity, dec!(26));
    Ok(())
}

#[test]
fn consecutive_inverse_ratios_cancel_exactly() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 30 @ 10\n\
         2024-02-01 SELL ABC 3 @ 12\n\
         2024-02-02 UNSPLIT ABC RATIO 3\n\
         2024-02-03 SPLIT ABC RATIO 3\n\
         2024-02-20 BUY ABC 3 @ 11\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(30));
    assert_eq!(
        report.tax_years[0].disposals[0].legs[0].allowable_cost,
        dec!(33)
    );
    Ok(())
}
