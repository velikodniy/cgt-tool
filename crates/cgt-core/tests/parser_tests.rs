#![allow(clippy::expect_used, clippy::panic)]

use cgt_core::models::Operation;
use cgt_core::parser::parse_file;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_parse_valid_buy() {
    let input = "2023-01-01 BUY AAPL 10 @ 150.00 FEES 5.00";
    let transactions = parse_file(input).expect("Failed to parse valid BUY transaction");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2023, 1, 1).expect("valid test date")
    );
    assert_eq!(tx.ticker, "AAPL");
    if let Operation::Buy {
        amount,
        price,
        fees,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(
            price.amount,
            Decimal::from_str("150.00").expect("valid decimal")
        );
        assert!(price.is_gbp());
        assert_eq!(
            fees.amount,
            Decimal::from_str("5.00").expect("valid decimal")
        );
        assert!(fees.is_gbp());
    } else {
        panic!("Expected Buy operation");
    }
}

#[test]
fn test_parse_dividend_with_tax_keyword() {
    let input = "2019-11-30 DIVIDEND GB00B3TYHH97 10 TOTAL 110.93 TAX 0";
    let transactions = parse_file(input).expect("Failed to parse DIVIDEND with TAX keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 11, 30).expect("valid test date")
    );
    assert_eq!(tx.ticker, "GB00B3TYHH97");
    if let Operation::Dividend {
        amount,
        total_value,
        tax_paid,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(
            total_value.amount,
            Decimal::from_str("110.93").expect("valid decimal")
        );
        assert!(total_value.is_gbp());
        assert_eq!(tax_paid.amount, Decimal::from(0));
        assert!(tax_paid.is_gbp());
    } else {
        panic!("Expected Dividend operation");
    }
}

#[test]
fn test_parse_capreturn_with_fees_keyword() {
    let input = "2019-05-31 CAPRETURN GB00B3TYHH97 10 TOTAL 149.75 FEES 0";
    let transactions = parse_file(input).expect("Failed to parse CAPRETURN with FEES keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 5, 31).expect("valid test date")
    );
    assert_eq!(tx.ticker, "GB00B3TYHH97");
    if let Operation::CapReturn {
        amount,
        total_value,
        fees,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(10));
        assert_eq!(
            total_value.amount,
            Decimal::from_str("149.75").expect("valid decimal")
        );
        assert!(total_value.is_gbp());
        assert_eq!(fees.amount, Decimal::from(0));
        assert!(fees.is_gbp());
    } else {
        panic!("Expected CapReturn operation");
    }
}

#[test]
fn test_parse_split_with_ratio_keyword() {
    let input = "2019-02-15 SPLIT FOO RATIO 2";
    let transactions = parse_file(input).expect("Failed to parse SPLIT with RATIO keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 2, 15).expect("valid test date")
    );
    assert_eq!(tx.ticker, "FOO");
    if let Operation::Split { ratio } = &tx.operation {
        assert_eq!(*ratio, Decimal::from(2));
    } else {
        panic!("Expected Split operation");
    }
}

#[test]
fn test_parse_unsplit_with_ratio_keyword() {
    let input = "2019-02-15 UNSPLIT FOO RATIO 2";
    let transactions = parse_file(input).expect("Failed to parse UNSPLIT with RATIO keyword");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2019, 2, 15).expect("valid test date")
    );
    assert_eq!(tx.ticker, "FOO");
    if let Operation::Unsplit { ratio } = &tx.operation {
        assert_eq!(*ratio, Decimal::from(2));
    } else {
        panic!("Expected Unsplit operation");
    }
}

// --- Multi-currency parsing tests ---

#[test]
fn test_parse_buy_without_currency_defaults_to_gbp() {
    let input = "2024-01-15 BUY AAPL 100 @ 150.00";
    let transactions = parse_file(input).expect("Failed to parse BUY without currency");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];

    if let Operation::Buy { price, fees, .. } = &tx.operation {
        assert!(price.is_gbp(), "Price should be GBP");
        assert_eq!(
            price.amount,
            Decimal::from_str("150.00").expect("valid decimal")
        );
        assert!(fees.is_gbp(), "Fees should be GBP");
    } else {
        panic!("Expected Buy operation");
    }
}

#[test]
fn test_parse_buy_with_gbp_currency_treated_as_default() {
    let input = "2024-01-15 BUY AAPL 100 @ 150.00 GBP";
    let transactions = parse_file(input).expect("Failed to parse BUY with explicit GBP");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];

    if let Operation::Buy { price, fees, .. } = &tx.operation {
        // Explicit GBP should be treated the same as default
        assert!(
            price.is_gbp(),
            "Explicit GBP price should be treated as GBP"
        );
        assert_eq!(
            price.amount,
            Decimal::from_str("150.00").expect("valid decimal")
        );
        assert!(fees.is_gbp());
    } else {
        panic!("Expected Buy operation");
    }
}

#[test]
fn test_parse_invalid_currency_code_errors() {
    let input = "2024-01-15 BUY AAPL 100 @ 150.00 ZZZ";
    let result = parse_file(input);
    assert!(result.is_err(), "Invalid currency code ZZZ should fail");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("ZZZ"), "Error should mention the invalid code");
}

#[test]
fn test_parse_split_not_confused_with_currency() {
    // SPLIT should not be confused with a currency code
    let input = "2024-01-15 SPLIT AAPL RATIO 4";
    let transactions = parse_file(input).expect("Failed to parse SPLIT");
    assert_eq!(transactions.len(), 1);

    if let Operation::Split { ratio } = &transactions[0].operation {
        assert_eq!(*ratio, Decimal::from(4));
    } else {
        panic!("Expected Split operation");
    }
}

#[test]
fn test_parse_fees_keyword_not_confused_with_currency() {
    // Make sure FEES keyword is parsed correctly, not as currency
    let input = "2024-01-15 BUY AAPL 100 @ 150.00 FEES 5.00";
    let transactions = parse_file(input).expect("Failed to parse");

    if let Operation::Buy { price, fees, .. } = &transactions[0].operation {
        assert!(price.is_gbp());
        assert_eq!(
            fees.amount,
            Decimal::from_str("5.00").expect("valid decimal")
        );
    } else {
        panic!("Expected Buy operation");
    }
}

// --- Optional clause tests ---

#[test]
fn test_parse_dividend_without_tax_clause() {
    // TAX clause should be optional, defaulting to 0 GBP
    let input = "2024-03-15 DIVIDEND VWRL 100 TOTAL 50.00";
    let transactions = parse_file(input).expect("Failed to parse DIVIDEND without TAX clause");
    assert_eq!(transactions.len(), 1);
    let tx = &transactions[0];
    assert_eq!(
        tx.date,
        NaiveDate::from_ymd_opt(2024, 3, 15).expect("valid test date")
    );
    assert_eq!(tx.ticker, "VWRL");
    if let Operation::Dividend {
        amount,
        total_value,
        tax_paid,
    } = &tx.operation
    {
        assert_eq!(*amount, Decimal::from(100));
        assert_eq!(
            total_value.amount,
            Decimal::from_str("50.00").expect("valid decimal")
        );
        assert!(total_value.is_gbp());
        // TAX defaults to 0 GBP when omitted
        assert_eq!(tax_paid.amount, Decimal::from(0));
        assert!(tax_paid.is_gbp());
    } else {
        panic!("Expected Dividend operation");
    }
}

#[test]
fn test_parse_dividend_without_tax_clause_with_currency() {
    // TAX clause should be optional even when total_value has a currency
    let input = "2024-03-15 DIVIDEND VWRL 100 TOTAL 50.00 USD";
    let transactions =
        parse_file(input).expect("Failed to parse DIVIDEND without TAX clause (with currency)");
    assert_eq!(transactions.len(), 1);
    if let Operation::Dividend {
        total_value,
        tax_paid,
        ..
    } = &transactions[0].operation
    {
        assert!(!total_value.is_gbp(), "total_value should be USD");
        // TAX defaults to 0 GBP when omitted
        assert_eq!(tax_paid.amount, Decimal::from(0));
        assert!(tax_paid.is_gbp());
    } else {
        panic!("Expected Dividend operation");
    }
}

#[test]
fn test_parse_sell_without_fees_clause() {
    // FEES clause should be optional for SELL, defaulting to 0 GBP
    let input = "2024-06-20 SELL AAPL 50 @ 180.00";
    let transactions = parse_file(input).expect("Failed to parse SELL without FEES clause");
    assert_eq!(transactions.len(), 1);
    if let Operation::Sell { amount, fees, .. } = &transactions[0].operation {
        assert_eq!(*amount, Decimal::from(50));
        // FEES defaults to 0 GBP when omitted
        assert_eq!(fees.amount, Decimal::from(0));
        assert!(fees.is_gbp());
    } else {
        panic!("Expected Sell operation");
    }
}

#[test]
fn test_parse_capreturn_without_fees_clause() {
    // FEES clause should be optional for CAPRETURN, defaulting to 0 GBP
    let input = "2024-04-01 CAPRETURN FUND 100 TOTAL 200.00";
    let transactions = parse_file(input).expect("Failed to parse CAPRETURN without FEES clause");
    assert_eq!(transactions.len(), 1);
    if let Operation::CapReturn { fees, .. } = &transactions[0].operation {
        // FEES defaults to 0 GBP when omitted
        assert_eq!(fees.amount, Decimal::from(0));
        assert!(fees.is_gbp());
    } else {
        panic!("Expected CapReturn operation");
    }
}
