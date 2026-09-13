use cgt::{CgtError, Config, calculate};
use rust_decimal_macros::dec;

#[test]
fn a_future_match_does_not_replenish_the_actual_holding() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 10 @ 10\n\
         2024-02-01 SELL ABC 10 @ 12\n\
         2024-02-02 SELL ABC 10 @ 13\n\
         2024-02-20 BUY ABC 10 @ 11\n",
    )?;
    let result = calculate(&transactions, None, None, &Config::embedded()?);
    assert!(
        matches!(result, Err(CgtError::UncoveredDisposal { held, attempted, .. }) if held == dec!(0) && attempted == dec!(10))
    );
    Ok(())
}

#[test]
fn consolidation_uses_actual_shares_instead_of_the_tax_pool()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(include_str!(
        "../../../tests/inputs/PhysicalConsolidation.cgt"
    ))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(7));
    assert_eq!(report.holdings[0].total_cost, dec!(105));
    Ok(())
}

#[test]
fn future_matches_do_not_hide_fractional_physical_consolidations()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 12 @ 10\n\
         2024-02-01 SELL ABC 1 @ 12\n\
         2024-02-02 UNSPLIT ABC RATIO 3\n\
         2024-02-20 BUY ABC 1 @ 30\n",
    )?;
    let result = calculate(&transactions, None, None, &Config::embedded()?);
    assert!(
        matches!(result, Err(CgtError::UnsplitIndivisibleHolding { holding, .. }) if holding == dec!(11))
    );
    Ok(())
}
