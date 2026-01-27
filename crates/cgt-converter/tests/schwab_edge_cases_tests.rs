use cgt_converter::schwab::{SchwabConverter, SchwabInput};
use cgt_converter::{BrokerConverter, ConvertError};

#[test]
fn test_empty_json() {
    let json = r#"{"BrokerageTransactions": []}"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // No warnings when no RSUs and no awards file
    assert_eq!(result.skipped_count, 0);
    assert!(
        result
            .cgt_content
            .contains("# Converted from Charles Schwab")
    );
}

#[test]
fn test_missing_required_field() {
    let json = r#"{
        "BrokerageTransactions": [
            {"Date": "04/25/2023", "Symbol": "XYZZ", "Description": "TEST"}
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConvertError::JsonError(_) | ConvertError::InvalidTransaction(_)
        ),
        "Expected JsonError or InvalidTransaction, got: {:?}",
        err
    );
}

#[test]
fn test_invalid_date_format() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "2023-04-25",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConvertError::InvalidDate(_)));
}

#[test]
fn test_invalid_amount_format() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "abc",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConvertError::InvalidAmount(_) | ConvertError::InvalidTransaction(_)
        ),
        "Expected InvalidAmount or InvalidTransaction, got: {:?}",
        err
    );
}

#[test]
fn test_buy_missing_quantity() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "--",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConvertError::InvalidTransaction(_)
    ));
}

#[test]
fn test_buy_missing_price() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "--",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConvertError::InvalidTransaction(_)
    ));
}

#[test]
fn test_rsu_with_invalid_awards_json() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "XYZZ",
                "Description": "RSU VEST",
                "Quantity": "67.2",
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$8443.47"
            }
        ]
    }"#;

    let invalid_json = "{ invalid json }";

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(invalid_json.to_string()),
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConvertError::JsonError(_)));
}

#[test]
fn test_rsu_missing_in_awards() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "XYZZ",
                "Description": "RSU VEST",
                "Quantity": "67.2",
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$8443.47"
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "BAR",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$340.00"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ConvertError::MissingFairMarketValue { .. }
    ));
}

#[test]
fn test_zero_expenses() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$0.00",
                "Amount": "-$1250.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // With zero expenses, they should be included
    assert!(result.cgt_content.contains("BUY XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_empty_expenses() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "--",
                "Amount": "-$1250.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Should still process with 0 expenses
    assert!(result.cgt_content.contains("BUY XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_unsupported_transaction_as_comment() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            },
            {
                "Date": "05/01/2023",
                "Action": "Wire Sent",
                "Symbol": "--",
                "Description": "WIRE TO BANK",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$1000.00"
            },
            {
                "Date": "05/15/2023",
                "Action": "Credit Interest",
                "Symbol": "--",
                "Description": "INTEREST EARNED",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$5.25"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should have comments for skipped transactions
    assert!(result.cgt_content.contains("# SKIPPED: Wire Sent"));
    assert!(result.cgt_content.contains("# SKIPPED: Credit Interest"));
    // skipped_count equals the number of warnings for skipped transactions.
    // Buys do not produce warnings, and missing awards is not warned when no RSUs are present.
    assert_eq!(result.skipped_count, 2);
}

#[test]
fn test_stock_split_as_comment() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            },
            {
                "Date": "06/01/2023",
                "Action": "Stock Split",
                "Symbol": "XYZZ",
                "Description": "2:1 SPLIT",
                "Quantity": "10",
                "Price": "--",
                "Fees & Comm": "--",
                "Amount": "--"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should have comment for stock split
    assert!(result.cgt_content.contains("# UNSUPPORTED: Stock split"));
    assert!(result.cgt_content.contains("SPLIT transaction manually"));
    // One unsupported transaction skipped (no warning for missing awards when no RSUs)
    assert_eq!(result.skipped_count, 1);
}

#[test]
fn test_multiple_dividends_same_day() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "07/15/2023",
                "Action": "Cash Dividend",
                "Symbol": "FOO",
                "Description": "DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$50.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "Cash Dividend",
                "Symbol": "FOO",
                "Description": "DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$25.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "NRA Tax Adj",
                "Symbol": "FOO",
                "Description": "TAX",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$11.25"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should combine tax with first dividend, second dividend has no tax
    assert!(result.cgt_content.contains("DIVIDEND FOO"));
    // Check that we have two dividend lines
    let dividend_count = result.cgt_content.matches("DIVIDEND FOO").count();
    assert_eq!(dividend_count, 2);
}

#[test]
fn test_large_numbers_with_commas() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "1,000",
                "Price": "$1,250.50",
                "Fees & Comm": "$49.95",
                "Amount": "-$1,250,549.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("BUY XYZZ 1000 @ 1250.50 USD"));
    assert!(result.cgt_content.contains("FEES 49.95 USD"));
}

#[test]
fn test_negative_amounts() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "$1245.05"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Amount sign doesn't affect SELL transaction parsing
    assert!(result.cgt_content.contains("SELL XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_whitespace_in_symbol() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": " XYZZ ",
                "Description": "TEST",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1254.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Symbol should be trimmed
    assert!(result.cgt_content.contains("BUY XYZZ 10"));
}

#[test]
fn test_fractional_shares() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "10.5",
                "Price": "$125.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$1317.45"
            },
            {
                "Date": "04/26/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "5.25",
                "Price": "$130.00",
                "Fees & Comm": "$2.50",
                "Amount": "$679.50"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("BUY XYZZ 10.5"));
    assert!(result.cgt_content.contains("SELL XYZZ 5.25"));
}

#[test]
fn test_very_precise_amounts() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "67.201495",
                "Price": "$125.6445",
                "Fees & Comm": "$0.1234",
                "Amount": "-$8443.4712"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("67.201495"));
    assert!(result.cgt_content.contains("125.6445"));
    assert!(result.cgt_content.contains("0.1234"));
}

// Share quantity precision tests
// Verify that parsing preserves exact decimal values without floating-point rounding

#[test]
fn test_quantity_precision_no_floating_point_errors() {
    // This test catches floating-point representation errors
    // 0.1 + 0.2 != 0.3 in floating point, but should work with Decimal
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "0.1",
                "Price": "$100.00",
                "Fees & Comm": "$0",
                "Amount": "-$10.00"
            },
            {
                "Date": "04/26/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "0.2",
                "Price": "$100.00",
                "Fees & Comm": "$0",
                "Amount": "-$20.00"
            },
            {
                "Date": "04/27/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "0.3",
                "Price": "$110.00",
                "Fees & Comm": "$0",
                "Amount": "$33.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // All quantities should be preserved exactly
    assert!(result.cgt_content.contains("BUY XYZZ 0.1 @"));
    assert!(result.cgt_content.contains("BUY XYZZ 0.2 @"));
    assert!(result.cgt_content.contains("SELL XYZZ 0.3 @"));
}

#[test]
fn test_rsu_quantity_precision_from_json_awards() {
    // Test that JSON parsing doesn't introduce floating-point rounding errors
    // in share quantities from equity awards
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "RSU VEST",
                "Quantity": "67.201495",
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$8443.47"
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$125.6445"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();
    // Quantity from CSV should be exact
    assert!(
        result.cgt_content.contains("BUY ACME 67.201495 @ 125.6445"),
        "Quantity and price should be preserved exactly: {}",
        result.cgt_content
    );
}

#[test]
fn test_extremely_small_fractional_shares() {
    // Very small fractional shares (e.g., from dividend reinvestment plans)
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "DRIP",
                "Quantity": "0.000001",
                "Price": "$100000.00",
                "Fees & Comm": "$0",
                "Amount": "-$0.10"
            },
            {
                "Date": "04/26/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "SELL",
                "Quantity": "0.000001",
                "Price": "$110000.00",
                "Fees & Comm": "$0",
                "Amount": "$0.11"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("0.000001"));
}

#[test]
fn test_large_quantity_no_precision_loss() {
    // Large quantities should not lose precision
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "TEST",
                "Quantity": "12345678.901234",
                "Price": "$0.00001",
                "Fees & Comm": "$0",
                "Amount": "-$123.46"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("12345678.901234"));
    assert!(result.cgt_content.contains("0.00001"));
}

#[test]
fn test_fmv_price_precision_preserved() {
    // FMV prices from awards should maintain full precision
    // Some awards have prices like $125.6445 (4 decimal places)
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "RSU VEST",
                "Quantity": "100",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$1234.567890"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();
    // Full precision should be preserved
    assert!(
        result.cgt_content.contains("1234.567890") || result.cgt_content.contains("1234.56789"),
        "FMV price precision should be preserved"
    );
}

// Additional edge cases

#[test]
fn test_dividend_negative_amount_handling() {
    // Some brokers report dividends with negative amounts for adjustments
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "07/15/2023",
                "Action": "Cash Dividend",
                "Symbol": "FOO",
                "Description": "DIVIDEND CORRECTION",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$10.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Should handle negative amount (abs value)
    assert!(result.cgt_content.contains("DIVIDEND FOO 10.00 USD"));
}

#[test]
fn test_tax_withholding_types() {
    // Both "NRA Tax Adj" and "NRA Withholding" should be recognized as tax
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "07/15/2023",
                "Action": "Cash Dividend",
                "Symbol": "FOO",
                "Description": "DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$100.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "NRA Tax Adj",
                "Symbol": "FOO",
                "Description": "TAX ADJUSTMENT",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$15.00"
            },
            {
                "Date": "07/20/2023",
                "Action": "Cash Dividend",
                "Symbol": "BAR",
                "Description": "DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$50.00"
            },
            {
                "Date": "07/20/2023",
                "Action": "NRA Withholding",
                "Symbol": "BAR",
                "Description": "WITHHOLDING TAX",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$7.50"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(
        result
            .cgt_content
            .contains("DIVIDEND FOO 100.00 USD TAX 15.00 USD")
    );
    assert!(
        result
            .cgt_content
            .contains("DIVIDEND BAR 50.00 USD TAX 7.50 USD")
    );
}
