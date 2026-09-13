use cgt::money::{Currency, CurrencyAmount, FxConversionError};
use cgt::{CgtError, Config, FxRates, TaxPeriod, calculate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn decimal_overflow_returns_an_error_at_each_engine_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let max = Decimal::MAX;
    let cases = [
        format!("2024-01-01 BUY ABC 2 @ {max}\n"),
        format!("2024-01-01 BUY ABC 1 @ {max} FEES 1\n"),
        format!("2024-01-01 BUY ABC 1 @ {max}\n2024-01-01 BUY ABC 1 @ 1\n"),
        format!("2024-01-01 BUY ABC {max} @ 0\n2024-01-01 BUY ABC 1 @ 0\n"),
        format!("2024-01-01 BUY ABC 1 @ 0 FEES {max}\n2024-01-01 BUY ABC 1 @ 0 FEES 1\n"),
        format!("2024-01-01 BUY ABC 1 @ {max}\n2024-01-02 BUY ABC 1 @ 1\n"),
        format!("2024-01-01 BUY ABC 1 @ {max}\n2024-01-02 ACCUMULATION ABC 1 TOTAL 1\n"),
        format!("2024-01-01 BUY ABC 1 @ {max}\n2024-01-02 CAPRETURN ABC 1 TOTAL 0 FEES {max}\n"),
        format!("2024-01-01 BUY ABC 1 @ {max}\n2024-03-01 SELL ABC 1 @ 0 FEES 1\n"),
        format!("2024-01-01 BUY ABC {max} @ 0\n2024-02-01 SPLIT ABC RATIO 2\n"),
    ];
    for input in cases {
        let transactions = cgt::dsl::parse(&input)?;
        let result = calculate(&transactions, None, None, &Config::embedded()?);
        assert!(
            matches!(result, Err(CgtError::ArithmeticOverflow { .. })),
            "{input}: {result:?}"
        );
    }
    Ok(())
}

#[test]
fn arithmetic_errors_name_the_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(&format!("2024-01-01 BUY ABC 2 @ {}\n", Decimal::MAX))?;
    let result = calculate(&transactions, None, None, &Config::embedded()?);
    assert!(
        matches!(&result, Err(CgtError::ArithmeticOverflow { operation }) if operation.contains("ABC") && operation.contains("2024-01-01"))
    );
    Ok(())
}

#[test]
fn maximum_representable_purchase_cost_stays_valid() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse(&format!("2024-01-01 BUY ABC 1 @ {}\n", Decimal::MAX))?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].total_cost, Decimal::MAX);
    Ok(())
}

#[test]
fn bundled_fx_conversion_reports_range_overflow() -> Result<(), Box<dyn std::error::Error>> {
    let date = chrono::NaiveDate::parse_from_str("2024-03-01", "%Y-%m-%d")?;
    let amount = CurrencyAmount::new(Decimal::MAX, Currency::KWD);
    assert!(matches!(
        amount.to_gbp(date, &FxRates::bundled()),
        Err(FxConversionError::ArithmeticOverflow { .. })
    ));
    let transactions = cgt::dsl::parse(&format!("2024-03-01 BUY ABC 1 @ {} KWD\n", Decimal::MAX))?;
    assert!(matches!(
        calculate(
            &transactions,
            None,
            Some(&FxRates::bundled()),
            &Config::embedded()?
        ),
        Err(CgtError::ArithmeticOverflow { .. })
    ));
    Ok(())
}

#[test]
fn tax_year_bounds_precede_end_year_arithmetic() {
    assert!(serde_json::from_str::<TaxPeriod>("\"65535/00\"").is_err());
    assert_eq!(cgt::format::format_tax_year(u16::MAX), "65535/36");
}

#[test]
fn exemptions_reject_negative_amounts_and_invalid_years() -> Result<(), Box<dyn std::error::Error>>
{
    let mut config = Config::embedded()?;
    let original = config.exemptions.clone();
    assert!(
        config
            .apply_overrides_toml("[exemptions]\n\"2024\" = -1\n")
            .is_err()
    );
    assert!(
        config
            .apply_overrides_toml("[exemptions]\n\"65535\" = 1\n")
            .is_err()
    );
    assert_eq!(config.exemptions, original);
    config.exemptions.insert(2024, dec!(-1));
    assert!(matches!(
        config.get_exemption(2024),
        Err(CgtError::ConfigError(_))
    ));
    Ok(())
}

#[test]
fn nonzero_quantities_below_decimal_precision_return_an_error()
-> Result<(), Box<dyn std::error::Error>> {
    let max = Decimal::MAX;
    let cases = [
        format!(
            "2024-01-01 BUY ABC 1 @ 1\n2024-02-01 SELL ABC 1 @ 2\n2024-02-02 SPLIT ABC RATIO {max}\n2024-02-20 BUY ABC 1 @ 1\n"
        ),
        "2024-01-01 BUY ABC 0.0000000000000000000000000001 @ 1\n2024-02-01 SPLIT ABC RATIO 0.1\n"
            .to_string(),
    ];
    for input in cases {
        let transactions = cgt::dsl::parse(&input)?;
        let result = calculate(&transactions, None, None, &Config::embedded()?);
        assert!(
            matches!(result, Err(CgtError::ArithmeticOverflow { operation }) if operation.contains("quantity") && operation.contains("precision"))
        );
    }
    Ok(())
}

#[test]
fn monetary_allocations_can_round_below_decimal_precision() -> Result<(), Box<dyn std::error::Error>>
{
    let transactions = cgt::dsl::parse(
        "2024-01-01 BUY ABC 3 @ 0 FEES 0.0000000000000000000000000001\n\
         2024-03-01 SELL ABC 1 @ 0\n\
         2024-03-02 SELL ABC 1 @ 0\n\
         2024-03-03 SELL ABC 1 @ 0\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    assert_eq!(report.holdings[0].quantity, dec!(0));
    assert_eq!(report.holdings[0].total_cost, dec!(0));
    Ok(())
}
