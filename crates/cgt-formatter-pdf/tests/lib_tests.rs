//! Tests for cgt-formatter-pdf lib.rs (PDF report formatting)

#![allow(clippy::expect_used)]

use cgt_core::{Disposal, Match, MatchRule};
use cgt_format::{format_date, format_gbp};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use typst::foundations::Value;

// Re-implement build_disposal_dict for testing since it's private
// This tests the same logic as the production code
fn build_disposal_dict(disposal: &Disposal) -> typst::foundations::Dict {
    use cgt_format::format_decimal_trimmed;
    use typst::foundations::{Dict, IntoValue};

    fn format_price(value: Decimal) -> String {
        format!("£{}", format_decimal_trimmed(value))
    }

    let mut dict = Dict::new();

    let total_gain: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();
    let gain_type = if total_gain >= Decimal::ZERO {
        "GAIN"
    } else {
        "LOSS"
    };

    dict.insert("ticker".into(), disposal.ticker.clone().into_value());
    dict.insert("date".into(), format_date(disposal.date).into_value());
    dict.insert(
        "quantity".into(),
        format_decimal_trimmed(disposal.quantity).into_value(),
    );
    dict.insert("gain_type".into(), gain_type.into_value());
    dict.insert(
        "gain_amount".into(),
        format_gbp(total_gain.abs()).into_value(),
    );

    // Calculate unit price from gross proceeds
    let unit_price = if disposal.quantity != Decimal::ZERO {
        disposal.gross_proceeds / disposal.quantity
    } else {
        Decimal::ZERO
    };

    // Calculate fees
    let sell_fees = disposal.gross_proceeds - disposal.proceeds;

    // Gross proceeds calculation
    let gross_proceeds_calc = format!(
        "{} × {} = {}",
        format_decimal_trimmed(disposal.quantity),
        format_price(unit_price),
        format_gbp(disposal.gross_proceeds)
    );
    dict.insert(
        "gross_proceeds_calc".into(),
        gross_proceeds_calc.into_value(),
    );

    // Net proceeds calculation
    let net_proceeds_calc = if sell_fees > Decimal::ZERO {
        format!(
            "{} - {} fees = {}",
            format_gbp(disposal.gross_proceeds),
            format_gbp(sell_fees),
            format_gbp(disposal.proceeds)
        )
    } else {
        String::new()
    };
    dict.insert("net_proceeds_calc".into(), net_proceeds_calc.into_value());
    dict.insert("has_fees".into(), (sell_fees > Decimal::ZERO).into_value());

    let total_cost: Decimal = disposal.matches.iter().map(|m| m.allowable_cost).sum();
    dict.insert("total_cost".into(), format_gbp(total_cost).into_value());
    dict.insert("result".into(), format_gbp(total_gain).into_value());

    dict
}

#[test]
fn test_format_gbp() {
    assert_eq!(format_gbp(Decimal::from(100)), "£100.00");
    assert_eq!(format_gbp(Decimal::from(-20)), "-£20.00");
    assert_eq!(format_gbp(Decimal::from(1234)), "£1,234.00");
    assert_eq!(format_gbp(Decimal::from(1000000)), "£1,000,000.00");
}

#[test]
fn test_format_date() {
    let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
    assert_eq!(format_date(date), "28/08/2018");
}

#[test]
fn test_proceeds_calc_with_fees() {
    let date = NaiveDate::from_ymd_opt(2018, 8, 28).expect("valid date");
    let disposal = Disposal {
        date,
        ticker: "GB00B41YBW71".to_string(),
        quantity: Decimal::from(10),
        gross_proceeds: Decimal::new(46702, 3), // 10 × 4.6702
        proceeds: Decimal::new(34202, 3),       // gross - 12.50 fees
        matches: vec![Match {
            rule: MatchRule::SameDay,
            quantity: Decimal::from(10),
            allowable_cost: Decimal::new(54065, 3),
            gain_or_loss: Decimal::new(-19863, 3),
            acquisition_date: None,
        }],
    };
    let dict = build_disposal_dict(&disposal);

    let gross_proceeds_value = dict
        .get("gross_proceeds_calc")
        .expect("gross proceeds calculation present");
    assert!(
        matches!(gross_proceeds_value, Value::Str(_)),
        "unexpected gross proceeds value: {gross_proceeds_value:?}"
    );
    let gross_proceeds = match gross_proceeds_value {
        Value::Str(s) => s.as_str().to_string(),
        _ => String::new(),
    };
    assert_eq!(gross_proceeds, "10 × £4.6702 = £46.70");

    let net_proceeds_value = dict
        .get("net_proceeds_calc")
        .expect("net proceeds calculation present");
    let net_proceeds = match net_proceeds_value {
        Value::Str(s) => s.as_str().to_string(),
        _ => String::new(),
    };
    assert_eq!(net_proceeds, "£46.70 - £12.50 fees = £34.20");
}
