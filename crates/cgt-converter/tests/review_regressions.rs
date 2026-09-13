use cgt_converter::BrokerConverter;
use cgt_converter::schwab::{SchwabConverter, SchwabInput};
use serde_json::json;

#[test]
fn overlapping_vests_keep_deposit_identity() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = json!({"BrokerageTransactions": [
        {"Date":"01/17/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"100"},
        {"Date":"01/19/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"200"}
    ]});
    let awards = json!({"Transactions": [
        {"Date":"01/17/2024","Action":"Deposit","Symbol":"FOO","Quantity":"100",
         "TransactionDetails":[{"Details":{"VestDate":"01/15/2024","VestFairMarketValue":"$10.00"}}]},
        {"Date":"01/19/2024","Action":"Deposit","Symbol":"FOO","Quantity":"200",
         "TransactionDetails":[{"Details":{"VestDate":"01/17/2024","VestFairMarketValue":"$20.00"}}]}
    ]});
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    })?;
    assert!(
        output
            .cgt_content
            .contains("2024-01-15 BUY FOO 100 @ 10.00 USD")
    );
    assert!(
        output
            .cgt_content
            .contains("2024-01-17 BUY FOO 200 @ 20.00 USD")
    );
    Ok(())
}

#[test]
fn ambiguous_vest_lookback_is_rejected() {
    let transactions = json!({"BrokerageTransactions": [
        {"Date":"01/19/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"100"}
    ]});
    let awards = json!({"Transactions": [
        {"Date":"01/15/2024","Action":"Lapse","Symbol":"FOO",
         "TransactionDetails":[{"Details":{"FairMarketValuePrice":"$10.00"}}]},
        {"Date":"01/17/2024","Action":"Lapse","Symbol":"FOO",
         "TransactionDetails":[{"Details":{"FairMarketValuePrice":"$20.00"}}]}
    ]});
    let result = SchwabConverter.convert(&SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    });
    assert!(
        matches!(result, Err(cgt_converter::ConvertError::InvalidTransaction(message)) if message.contains("Ambiguous awards"))
    );
}

#[test]
fn dividend_corrections_preserve_signed_income() -> Result<(), Box<dyn std::error::Error>> {
    for amounts in [
        vec!["100.00", "-10.00"],
        vec!["-100.00", "100.00", "100.00"],
    ] {
        let expected: rust_decimal::Decimal = if amounts.len() == 2 {
            90.into()
        } else {
            100.into()
        };
        let rows: Vec<_> = amounts
            .into_iter()
            .map(|amount| {
                json!({
                    "Date":"07/15/2023", "Action":"Cash Dividend", "Symbol":"FOO", "Amount":amount
                })
            })
            .collect();
        let output = SchwabConverter.convert(&SchwabInput {
            transactions_json: json!({"BrokerageTransactions":rows}).to_string(),
            awards_json: None,
        })?;
        let transactions = cgt::dsl::parse(&output.cgt_content)?;
        let total: rust_decimal::Decimal = transactions
            .iter()
            .filter_map(|transaction| {
                if let cgt::model::Operation::Dividend { total_value, .. } = &transaction.operation
                {
                    Some(total_value.amount)
                } else {
                    None
                }
            })
            .sum();
        assert_eq!(total, expected);
    }
    Ok(())
}

#[test]
fn cancellation_keeps_the_corrected_fee() -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<_> = [
        ("Sell", "1.00"),
        ("Cancel Sell", "10.00"),
        ("Sell", "10.00"),
    ]
    .into_iter()
    .map(|(action, fees)| {
        json!({
            "Date":"07/15/2023", "Action":action, "Symbol":"FOO", "Quantity":"100",
            "Price":"50.00", "Fees & Comm":fees
        })
    })
    .collect();
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: json!({"BrokerageTransactions":rows}).to_string(),
        awards_json: None,
    })?;
    assert_eq!(output.cgt_content.matches("SELL FOO").count(), 1);
    assert!(output.cgt_content.contains("FEES 1.00 USD"));
    Ok(())
}

#[test]
fn cancellation_without_fees_rejects_distinct_candidates() {
    let rows: Vec<_> = [
        ("Sell", Some("1.00")),
        ("Cancel Sell", None),
        ("Sell", Some("10.00")),
    ]
    .into_iter()
    .map(|(action, fees)| {
        json!({
            "Date":"07/15/2023", "Action":action, "Symbol":"FOO", "Quantity":"100",
            "Price":"50.00", "Fees & Comm":fees
        })
    })
    .collect();
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: json!({"BrokerageTransactions":rows}).to_string(),
        awards_json: None,
    });
    assert!(
        matches!(output, Err(cgt_converter::ConvertError::InvalidTransaction(message)) if message.contains("Ambiguous Cancel Sell"))
    );
}
