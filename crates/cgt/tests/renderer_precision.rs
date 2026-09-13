use cgt::{Config, calculate, dsl, format::plain, report::Holding};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn holding_json_preserves_the_average_before_total_cost_rounding()
-> Result<(), Box<dyn std::error::Error>> {
    let input = include_str!("../../../tests/renderers/holding_precision.cgt");
    let transactions = dsl::parse(input)?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let json = serde_json::to_value(&report)?;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/renderers/holding_precision.json"
    ))?;
    assert_eq!(json, expected);
    assert_eq!(json["holdings"][0]["total_cost"], "10.00");
    assert_eq!(json["holdings"][0]["average_cost"], "100.005");
    assert!(plain::render(&report).contains("0.1 units at £100.01 avg cost"));
    Ok(())
}

#[test]
fn holding_average_handles_zero_quantity_and_reports_overflow()
-> Result<(), Box<dyn std::error::Error>> {
    let mut holding = Holding {
        ticker: "LIMIT".into(),
        quantity: Decimal::ZERO,
        total_cost: Decimal::MAX,
    };
    assert_eq!(serde_json::to_value(&holding)?["average_cost"], "0");
    holding.quantity = dec!(0.1);
    assert!(serde_json::to_value(&holding).is_err());
    Ok(())
}

#[test]
fn plain_sale_fees_reconcile_displayed_proceeds() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = dsl::parse(
        "2024-01-01 BUY FEE 1 @ 0.5\n\
         2024-06-01 SELL FEE 1 @ 1.004 FEES 0.005\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let text = plain::render(&report);
    assert!(
        text.contains("Net Proceeds: £1.00 - £0.00 fees = £1.00"),
        "{text}"
    );
    Ok(())
}
