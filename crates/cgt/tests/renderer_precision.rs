use cgt::{Config, calculate, dsl, format::plain, report::Holding};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn holding_json_preserves_exact_cost_for_derived_unit_values()
-> Result<(), Box<dyn std::error::Error>> {
    let input = include_str!("../../../tests/renderers/holding_precision.cgt");
    let transactions = dsl::parse(input)?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let json = serde_json::to_value(&report)?;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/renderers/holding_precision.json"
    ))?;
    assert_eq!(json, expected);
    assert_eq!(json["holdings"][0]["total_cost"], "10.0005");
    assert!(json["holdings"][0].get("average_cost").is_none());
    assert!(
        plain::render(&cgt::report_display::prepare(&report)?)
            .contains("0.1 units at £100.01 avg cost")
    );
    Ok(())
}

#[test]
fn holding_serialization_preserves_values_without_division()
-> Result<(), Box<dyn std::error::Error>> {
    let mut holding = Holding {
        ticker: "LIMIT".into(),
        quantity: Decimal::ZERO,
        total_cost: Decimal::MAX,
    };
    for quantity in [Decimal::ZERO, dec!(0.1)] {
        holding.quantity = quantity;
        let json = serde_json::to_value(&holding)?;
        assert_eq!(json["quantity"], quantity.to_string());
        assert_eq!(json["total_cost"], Decimal::MAX.to_string());
        assert!(json.get("average_cost").is_none());
    }
    Ok(())
}

#[test]
fn plain_sale_fees_reconcile_displayed_proceeds() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = dsl::parse(
        "2024-01-01 BUY FEE 1 @ 0.5\n\
         2024-06-01 SELL FEE 1 @ 1.004 FEES 0.005\n",
    )?;
    let report = calculate(&transactions, None, None, &Config::embedded()?)?;
    let text = plain::render(&cgt::report_display::prepare(&report)?);
    assert!(
        text.contains("Net Proceeds: £1.00 - £0.00 fees = £1.00"),
        "{text}"
    );
    Ok(())
}
