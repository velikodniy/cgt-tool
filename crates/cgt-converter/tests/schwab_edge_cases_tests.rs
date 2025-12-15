use cgt_converter::schwab::{AwardsFormat, SchwabConverter, SchwabInput};
use cgt_converter::{BrokerConverter, ConvertError};

#[test]
fn test_empty_csv() {
    let csv = "Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount\n";

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    assert_eq!(result.skipped_count, 0);
    assert!(
        result
            .cgt_content
            .contains("# Converted from Charles Schwab")
    );
}

#[test]
fn test_missing_required_column() {
    let csv = "Date,Action,Symbol,Description\n04/25/2023,Buy,XYZZ,TEST\n";

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    // Missing column can manifest as CsvError, MissingColumn, or InvalidTransaction
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConvertError::MissingColumn(_)
                | ConvertError::CsvError(_)
                | ConvertError::InvalidTransaction(_)
        ),
        "Expected MissingColumn, CsvError, or InvalidTransaction, got: {:?}",
        err
    );
}

#[test]
fn test_invalid_date_format() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
2023-04-25,Buy,XYZZ,TEST,10,$125.00,$4.95,-$1254.95
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConvertError::InvalidDate(_)));
}

#[test]
fn test_invalid_amount_format() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,abc,$125.00,$4.95,-$1254.95
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    // CSV parser might return CsvError or we might get InvalidAmount - both are acceptable
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConvertError::InvalidAmount(_)
                | ConvertError::CsvError(_)
                | ConvertError::InvalidTransaction(_)
        ),
        "Expected InvalidAmount, CsvError, or InvalidTransaction, got: {:?}",
        err
    );
}

#[test]
fn test_buy_missing_quantity() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,--,$125.00,$4.95,-$1254.95
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10,--,$4.95,-$1254.95
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Stock Plan Activity,XYZZ,RSU VEST,67.2,--,--,$8443.47
"#;

    let invalid_json = "{ invalid json }";

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: Some(invalid_json.to_string()),
        awards_format: Some(AwardsFormat::Json),
    };

    let result = converter.convert(&input);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConvertError::JsonError(_)));
}

#[test]
fn test_rsu_missing_in_awards() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Stock Plan Activity,XYZZ,RSU VEST,67.2,--,--,$8443.47
"#;

    let awards = r#"{
        "EquityAwards": [
            {
                "Symbol": "BAR",
                "EventDate": "04/25/2023",
                "FairMarketValuePrice": "$340.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: Some(awards.to_string()),
        awards_format: Some(AwardsFormat::Json),
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10,$125.00,$0.00,-$1250.00
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    // With zero expenses, they should be included
    assert!(result.cgt_content.contains("BUY XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_empty_expenses() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10,$125.00,--,-$1250.00
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    // Should still process with 0 expenses
    assert!(result.cgt_content.contains("BUY XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_unsupported_transaction_as_comment() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10,$125.00,$4.95,-$1254.95
05/01/2023,Wire Sent,--,WIRE TO BANK,--,--,--,-$1000.00
05/15/2023,Credit Interest,--,INTEREST EARNED,--,--,--,$5.25
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should have comments for skipped transactions
    assert!(result.cgt_content.contains("# SKIPPED: Wire Sent"));
    assert!(result.cgt_content.contains("# SKIPPED: Credit Interest"));
    assert_eq!(result.skipped_count, 2);
}

#[test]
fn test_stock_split_as_comment() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10,$125.00,$4.95,-$1254.95
06/01/2023,Stock Split,XYZZ,2:1 SPLIT,10,--,--,--
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should have comment for stock split
    assert!(result.cgt_content.contains("# UNSUPPORTED: Stock split"));
    assert!(result.cgt_content.contains("SPLIT transaction manually"));
    assert_eq!(result.skipped_count, 1);
}

#[test]
fn test_multiple_dividends_same_day() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
07/15/2023,Cash Dividend,FOO,DIVIDEND,--,--,--,$50.00
07/15/2023,Cash Dividend,FOO,DIVIDEND,--,--,--,$25.00
07/15/2023,NRA Tax Adj,FOO,TAX,--,--,--,-$11.25
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,"1,000","$1,250.50",$49.95,"-$1,250,549.95"
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("BUY XYZZ 1000 @ 1250.50 USD"));
    assert!(result.cgt_content.contains("EXPENSES 49.95 USD"));
}

#[test]
fn test_negative_amounts() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Sell,XYZZ,TEST,10,$125.00,$4.95,$1245.05
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    // Amount sign doesn't affect SELL transaction parsing
    assert!(result.cgt_content.contains("SELL XYZZ 10 @ 125.00 USD"));
}

#[test]
fn test_whitespace_in_symbol() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy, XYZZ ,TEST,10,$125.00,$4.95,-$1254.95
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    // Symbol should be trimmed
    assert!(result.cgt_content.contains("BUY XYZZ 10"));
}

#[test]
fn test_fractional_shares() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,10.5,$125.00,$4.95,-$1317.45
04/26/2023,Sell,XYZZ,TEST,5.25,$130.00,$2.50,$679.50
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("BUY XYZZ 10.5"));
    assert!(result.cgt_content.contains("SELL XYZZ 5.25"));
}

#[test]
fn test_very_precise_amounts() {
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,67.201495,$125.6445,$0.1234,-$8443.4712
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,0.1,$100.00,$0,-$10.00
04/26/2023,Buy,XYZZ,TEST,0.2,$100.00,$0,-$20.00
04/27/2023,Sell,XYZZ,TEST,0.3,$110.00,$0,$33.00
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Stock Plan Activity,ACME,RSU VEST,67.201495,--,--,$8443.47
"#;

    let awards = r#"{
        "EquityAwards": [
            {
                "Symbol": "ACME",
                "EventDate": "04/25/2023",
                "FairMarketValuePrice": "$125.6445"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: Some(awards.to_string()),
        awards_format: Some(AwardsFormat::Json),
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,DRIP,0.000001,$100000.00,$0,-$0.10
04/26/2023,Sell,XYZZ,SELL,0.000001,$110000.00,$0,$0.11
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("0.000001"));
}

#[test]
fn test_large_quantity_no_precision_loss() {
    // Large quantities should not lose precision
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Buy,XYZZ,TEST,12345678.901234,$0.00001,$0,-$123.46
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("12345678.901234"));
    assert!(result.cgt_content.contains("0.00001"));
}

#[test]
fn test_fmv_price_precision_preserved() {
    // FMV prices from awards should maintain full precision
    // Some awards have prices like $125.6445 (4 decimal places)
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
04/25/2023,Stock Plan Activity,ACME,RSU VEST,100,--,--,--
"#;

    let awards = r#"{
        "EquityAwards": [
            {
                "Symbol": "ACME",
                "EventDate": "04/25/2023",
                "FairMarketValuePrice": "$1234.567890"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: Some(awards.to_string()),
        awards_format: Some(AwardsFormat::Json),
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
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
07/15/2023,Cash Dividend,FOO,DIVIDEND CORRECTION,--,--,--,-$10.00
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
    };

    let result = converter.convert(&input).unwrap();
    // Should handle negative amount (abs value)
    assert!(result.cgt_content.contains("DIVIDEND FOO 10.00 USD"));
}

#[test]
fn test_tax_withholding_types() {
    // Both "NRA Tax Adj" and "NRA Withholding" should be recognized as tax
    let csv = r#"Date,Action,Symbol,Description,Quantity,Price,Fees & Comm,Amount
07/15/2023,Cash Dividend,FOO,DIVIDEND,--,--,--,$100.00
07/15/2023,NRA Tax Adj,FOO,TAX ADJUSTMENT,--,--,--,-$15.00
07/20/2023,Cash Dividend,BAR,DIVIDEND,--,--,--,$50.00
07/20/2023,NRA Withholding,BAR,WITHHOLDING TAX,--,--,--,-$7.50
"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_csv: csv.to_string(),
        awards_content: None,
        awards_format: None,
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
