use cgt::{CgtError, Config, calculate, dsl};
use rust_decimal::Decimal;

#[test]
fn report_aggregation_returns_errors_when_totals_exceed_decimal_range()
-> Result<(), Box<dyn std::error::Error>> {
    for input in [
        "2024-01-01 BUY A 1 @ 0\n2024-01-01 BUY B 1 @ 0\n\
         2024-06-01 SELL A 1 @ 79228162514264337593543950335\n\
         2024-06-01 SELL B 1 @ 79228162514264337593543950335\n",
        "2024-06-01 DIVIDEND A TOTAL 79228162514264337593543950335\n\
         2024-06-02 DIVIDEND A TOTAL 79228162514264337593543950335\n",
        "2024-01-01 BUY A 2 @ 1\n2024-06-01 SELL A 2 @ 0\n\
         2024-06-02 BUY A 1 @ 79228162514264337593543950335\n\
         2024-06-03 BUY A 1 @ 79228162514264337593543950335\n",
    ] {
        let transactions = dsl::parse(input)?;
        assert!(matches!(
            calculate(&transactions, None, None, &Config::embedded()?),
            Err(CgtError::ArithmeticOverflow { .. })
        ));
    }
    Ok(())
}

#[test]
fn a_maximum_loss_needs_no_subtraction_of_the_exemption() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = dsl::parse(
        "2024-01-01 BUY A 1 @ 79228162514264337593543950335\n\
         2024-06-01 SELL A 1 @ 0\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let year = report.tax_years.first().ok_or("missing tax year")?;
    assert_eq!(year.total_loss, Decimal::MAX);
    assert_eq!(year.taxable_gain, Decimal::ZERO);
    assert!(!cgt::plain::render(&cgt::report_display::prepare(&report)?).is_empty());
    serde_json::to_value(&report)?;
    Ok(())
}

#[test]
fn unrepresentable_unit_cost_returns_an_error_before_rendering()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = dsl::parse(
        "2024-01-01 BUY A 0.0000000000000000000000000001 @ 0\n\
         2024-06-01 ACCUMULATION A 1 TOTAL 79228162514264337593543950335\n",
    )?;
    assert!(matches!(
        calculate(&transactions, None, None, &Config::embedded()?),
        Err(CgtError::ArithmeticOverflow { .. })
    ));
    Ok(())
}

#[test]
fn a_negative_exemption_is_not_a_missing_exemption() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::embedded()?;
    config.exemptions.insert(2024, -Decimal::ONE);
    config.allow_missing_exemption = true;
    let transactions = dsl::parse("2024-06-01 DIVIDEND A TOTAL 1\n")?;
    assert!(calculate(&transactions, Some(2024), None, &config).is_err());
    Ok(())
}
