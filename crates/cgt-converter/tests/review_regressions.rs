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

#[test]
fn dividends_require_a_string_amount() {
    for amount in [json!(null), json!(100), json!(""), json!("--")] {
        let output = SchwabConverter.convert(&SchwabInput {
            transactions_json: json!({"BrokerageTransactions":[{
                "Date":"07/15/2023", "Action":"Cash Dividend", "Symbol":"FOO", "Amount":amount
            }]})
            .to_string(),
            awards_json: None,
        });
        assert!(matches!(
            output,
            Err(cgt_converter::ConvertError::InvalidTransaction(_))
        ));
    }
}

#[test]
fn withholding_credits_reduce_tax_and_unmatched_rows_are_reported()
-> Result<(), Box<dyn std::error::Error>> {
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: json!({"BrokerageTransactions":[
            {"Date":"07/15/2023", "Action":"Cash Dividend", "Symbol":"FOO", "Amount":"100.00"},
            {"Date":"07/15/2023", "Action":"NRA Withholding", "Symbol":"FOO", "Amount":"-15.00"},
            {"Date":"07/15/2023", "Action":"NRA Tax Adj", "Symbol":"FOO", "Amount":"5.00"},
            {"Date":"07/16/2023", "Action":"NRA Withholding", "Symbol":"FOO", "Amount":"-2.00"},
            {"Date":"07/17/2023", "Action":"NRA Withholding", "Symbol":"", "Amount":"-3.00"}
        ]})
        .to_string(),
        awards_json: None,
    })?;
    assert!(output.cgt_content.contains("TAX 10.00 USD"));
    assert_eq!(output.warnings.len(), 2);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.contains("2023-07-16"))
    );
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.contains("missing symbol"))
    );
    Ok(())
}

#[test]
fn unmatched_tax_credit_is_rejected() {
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: json!({"BrokerageTransactions":[
            {"Date":"07/15/2023", "Action":"Cash Dividend", "Symbol":"FOO", "Amount":"100.00"},
            {"Date":"07/15/2023", "Action":"NRA Tax Adj", "Symbol":"FOO", "Amount":"5.00"}
        ]})
        .to_string(),
        awards_json: None,
    });
    assert!(
        matches!(output, Err(cgt_converter::ConvertError::InvalidTransaction(message)) if message.contains("Tax credits exceed"))
    );
}

#[test]
fn broker_symbols_round_trip_through_the_dsl() -> Result<(), Box<dyn std::error::Error>> {
    for symbol in ["BRK/B", "BRK.B", "FOO-B", "FOO_B"] {
        let output = SchwabConverter.convert(&SchwabInput {
            transactions_json: json!({"BrokerageTransactions":[{
                "Date":"07/15/2023", "Action":"Buy", "Symbol":symbol, "Quantity":"1", "Price":"100"
            }]})
            .to_string(),
            awards_json: None,
        })?;
        let transactions = cgt::dsl::parse(&output.cgt_content)?;
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].ticker, symbol);
        assert_eq!(
            cgt::dsl::parse(&cgt::dsl::serialize(&transactions))?,
            transactions
        );
    }
    Ok(())
}

#[test]
fn unsupported_descriptions_cannot_create_transactions() -> Result<(), Box<dyn std::error::Error>> {
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: json!({"BrokerageTransactions":[{
            "Date":"07/15/2023", "Action":"Unknown", "Symbol":"FOO",
            "Description":"text\r2023-07-15 BUY FOO 100 @ 1 USD\n# end"
        }]})
        .to_string(),
        awards_json: None,
    })?;
    assert!(cgt::dsl::parse(&output.cgt_content)?.is_empty());
    Ok(())
}

#[test]
fn invalid_symbols_fail_during_conversion() {
    for symbol in ["BRK B", "FOO#", "FOO\nBAR", ".FOO"] {
        let output = SchwabConverter.convert(&SchwabInput {
            transactions_json: json!({"BrokerageTransactions":[{
                "Date":"07/15/2023", "Action":"Buy", "Symbol":symbol, "Quantity":"1", "Price":"100"
            }]})
            .to_string(),
            awards_json: None,
        });
        assert!(matches!(
            output,
            Err(cgt_converter::ConvertError::InvalidTransaction(_))
        ));
    }
}

#[test]
fn conflicting_award_values_remain_ambiguous_without_a_deposit() {
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json:json!({"BrokerageTransactions":[{
            "Date":"01/17/2024", "Action":"Stock Plan Activity", "Symbol":"FOO", "Quantity":"100"
        }]}).to_string(),
        awards_json:Some(json!({"Transactions":[
            {"Date":"01/15/2024", "Action":"Lapse", "Symbol":"FOO", "TransactionDetails":[{"Details":{"FairMarketValuePrice":"10"}}]},
            {"Date":"01/15/2024", "Action":"Lapse", "Symbol":"FOO", "TransactionDetails":[{"Details":{"FairMarketValuePrice":"20"}}]}
        ]}).to_string()),
    });
    assert!(
        matches!(output, Err(cgt_converter::ConvertError::InvalidTransaction(message)) if message.contains("Ambiguous awards"))
    );
}

#[test]
fn conversion_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let input = SchwabInput {
        transactions_json: json!({"BrokerageTransactions":[{
            "Date":"07/15/2023", "Action":"Buy", "Symbol":"FOO", "Quantity":"1", "Price":"100"
        }]})
        .to_string(),
        awards_json: None,
    };
    let first = SchwabConverter.convert(&input)?;
    let second = SchwabConverter.convert(&input)?;
    assert_eq!(first, second);
    assert!(!first.cgt_content.contains("# Converted:"));
    Ok(())
}
