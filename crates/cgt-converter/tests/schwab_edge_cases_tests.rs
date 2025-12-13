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
