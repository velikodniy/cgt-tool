use cgt_converter::BrokerConverter;
use cgt_converter::schwab::{SchwabConverter, SchwabInput};

#[test]
fn test_basic_buy_sell() {
    let json = include_str!("fixtures/schwab/transactions_basic.json");

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    assert!(result.cgt_content.contains("2023-04-25 BUY XYZZ"));
    assert!(result.cgt_content.contains("@ 125.50 USD"));
    assert!(result.cgt_content.contains("FEES 4.95 USD"));
    assert!(result.cgt_content.contains("2023-05-10 SELL XYZZ"));
    assert!(result.cgt_content.contains("@ 130.00 USD"));
    assert!(result.cgt_content.contains("FEES 2.50 USD"));

    // Should contain header
    assert!(
        result
            .cgt_content
            .contains("# Converted from Charles Schwab")
    );
}

#[test]
fn test_rsu_vesting_with_awards() {
    let json = include_str!("fixtures/schwab/transactions_rsu.json");
    let awards = include_str!("fixtures/schwab/awards.json");

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // Should contain RSU vesting as BUY with FMV price
    assert!(
        result
            .cgt_content
            .contains("# RSU Vesting - FMV from awards file")
    );
    assert!(
        result
            .cgt_content
            .contains("2023-04-25 BUY XYZZ 67.2 @ 125.6445 USD")
    );

    // Should contain the sell
    assert!(
        result
            .cgt_content
            .contains("2023-04-25 SELL XYZZ 20.2 @ 125.6445 USD")
    );

    // Should mention both source files
    assert!(result.cgt_content.contains("awards.json"));
}

#[test]
fn test_rsu_vesting_uses_vest_fields() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/28/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            }
        ]
    }"#;
    let awards = include_str!("fixtures/schwab/awards_vest.json");

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    assert!(
        result
            .cgt_content
            .contains("2023-04-25 BUY XYZZ 10 @ 125.50 USD")
    );
}

#[test]
fn test_rsu_vesting_without_awards_fails() {
    let json = include_str!("fixtures/schwab/transactions_rsu.json");

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input);

    // Should fail because RSU needs FMV from awards file
    assert!(result.is_err());
}

#[test]
fn test_dividend_with_tax() {
    let json = include_str!("fixtures/schwab/transactions_dividend.json");

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should combine dividend with tax
    assert!(
        result
            .cgt_content
            .contains("2023-07-15 DIVIDEND FOO 50.00 USD TAX 7.50 USD")
    );
}

#[test]
fn test_date_formats() {
    // Test "as of" date format
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "as of 04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.50",
                "Fees & Comm": "$4.95",
                "Amount": "-$1259.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Should parse "as of" date correctly
    assert!(result.cgt_content.contains("2023-04-25 BUY XYZZ"));
}

#[test]
fn test_chronological_sorting() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "06/15/2023",
                "Action": "Buy",
                "Symbol": "FOO",
                "Description": "FOO INC",
                "Quantity": "5",
                "Price": "$150.00",
                "Fees & Comm": "$2.00",
                "Amount": "-$752.00"
            },
            {
                "Date": "04/10/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$1254.00"
            },
            {
                "Date": "05/20/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "5",
                "Price": "$130.00",
                "Fees & Comm": "$2.00",
                "Amount": "$648.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Find the positions of each transaction
    let goog_buy_pos = result.cgt_content.find("2023-04-10 BUY XYZZ").unwrap();
    let goog_sell_pos = result.cgt_content.find("2023-05-20 SELL XYZZ").unwrap();
    let aapl_buy_pos = result.cgt_content.find("2023-06-15 BUY FOO").unwrap();

    // Should be sorted chronologically (oldest first)
    assert!(goog_buy_pos < goog_sell_pos);
    assert!(goog_sell_pos < aapl_buy_pos);
}

#[test]
fn test_skipped_transactions() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$1254.00"
            },
            {
                "Date": "05/01/2023",
                "Action": "Wire Sent",
                "Symbol": "--",
                "Description": "WIRE TO EXTERNAL ACCOUNT",
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

    // Skipped transactions are recorded as comments (no warnings when no RSUs)
    assert_eq!(result.skipped_count, 2);
    assert!(result.warnings.is_empty());

    // Output should contain the Buy and comments for skipped transactions
    assert!(result.cgt_content.contains("BUY XYZZ"));
    // Unsupported transactions are now added as comments
    assert!(result.cgt_content.contains("# SKIPPED: Wire Sent"));
    assert!(result.cgt_content.contains("# SKIPPED: Credit Interest"));
}

// ===========================================
// Date Format Integration Tests
// ===========================================

#[test]
fn test_date_format_as_of_suffix() {
    // Real-world format: "MM/DD/YYYY as of MM/DD/YYYY"
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "03/20/2024 as of 03/19/2024",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.50",
                "Fees & Comm": "$4.95",
                "Amount": "-$1259.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Should use the "as of" date (03/19/2024)
    assert!(result.cgt_content.contains("2024-03-19 BUY XYZZ"));
}

#[test]
fn test_date_format_single_digit_month() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "1/5/2024 as of 1/4/2024",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.50",
                "Fees & Comm": "$4.95",
                "Amount": "-$1259.95"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("2024-01-04 BUY XYZZ"));
}

#[test]
fn test_mixed_date_formats() {
    // Mix of different date formats in the same file
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$1254.00"
            },
            {
                "Date": "as of 05/10/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "5",
                "Price": "$130.00",
                "Fees & Comm": "$2.00",
                "Amount": "$648.00"
            },
            {
                "Date": "06/15/2023 as of 06/14/2023",
                "Action": "Buy",
                "Symbol": "FOO",
                "Description": "FOO INC",
                "Quantity": "20",
                "Price": "$150.00",
                "Fees & Comm": "$5.00",
                "Amount": "-$3005.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("2023-04-25 BUY XYZZ"));
    assert!(result.cgt_content.contains("2023-05-10 SELL XYZZ"));
    assert!(result.cgt_content.contains("2023-06-14 BUY FOO"));
}

// ===========================================
// RSU Vesting Scenarios
// ===========================================

#[test]
fn test_rsu_vest_and_sell_to_cover() {
    // Typical RSU scenario: vest shares, immediately sell some to cover taxes
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "100",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            },
            {
                "Date": "04/25/2023",
                "Action": "Sell",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "30",
                "Price": "$50.25",
                "Fees & Comm": "$0.05",
                "Amount": "$1507.45"
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$50.25"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // Should have BUY for vesting
    assert!(
        result
            .cgt_content
            .contains("2023-04-25 BUY ACME 100 @ 50.25 USD")
    );
    // Should have SELL for tax withholding
    assert!(
        result
            .cgt_content
            .contains("2023-04-25 SELL ACME 30 @ 50.25 USD")
    );
}

#[test]
fn test_rsu_multiple_vests_same_day() {
    // Multiple RSU grants vesting on the same day
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "ACME CORP RSU 1",
                "Quantity": "50",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            },
            {
                "Date": "04/25/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "ACME CORP RSU 2",
                "Quantity": "75",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            },
            {
                "Date": "04/25/2023",
                "Action": "Sell",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "40",
                "Price": "$50.25",
                "Fees & Comm": "$0.05",
                "Amount": "$2009.95"
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$50.25"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // Should have two BUY transactions for the two vests
    let buy_count = result.cgt_content.matches("BUY ACME").count();
    assert_eq!(buy_count, 2);
    assert!(result.cgt_content.contains("BUY ACME 50 @ 50.25 USD"));
    assert!(result.cgt_content.contains("BUY ACME 75 @ 50.25 USD"));
}

#[test]
fn test_rsu_vest_date_used_for_cgt_acquisition() {
    // Settlement date in transactions (04/28) differs from vest date in awards (04/25)
    // Per HMRC guidance (CG14250, ERSM20192), the vest date should be used as CGT acquisition date
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/28/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "100",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            }
        ]
    }"#;

    // Award/vest date is 04/25 but transaction settlement is 04/28 (T+2 settlement)
    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/25/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$50.25"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();
    // Should use VEST date (04/25) not settlement date (04/28) for CGT acquisition
    assert!(
        result
            .cgt_content
            .contains("2023-04-25 BUY ACME 100 @ 50.25 USD"),
        "Expected vest date 2023-04-25 but got: {}",
        result.cgt_content
    );
    // Should NOT use the settlement date
    assert!(
        !result.cgt_content.contains("2023-04-28 BUY ACME"),
        "Settlement date should not be used for RSU acquisition"
    );
}

// ===========================================
// Dividend Scenarios
// ===========================================

#[test]
fn test_multiple_dividend_types() {
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
                "Action": "Qualified Dividend",
                "Symbol": "BAR",
                "Description": "QUALIFIED DIV",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$30.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "Short Term Cap Gain",
                "Symbol": "ETFX",
                "Description": "SHORT TERM GAIN",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$10.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "Long Term Cap Gain",
                "Symbol": "ETFX",
                "Description": "LONG TERM GAIN",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$20.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    assert!(result.cgt_content.contains("DIVIDEND FOO 50.00 USD"));
    assert!(result.cgt_content.contains("DIVIDEND BAR 30.00 USD"));
    // Capital gains treated as dividends
    assert!(result.cgt_content.contains("DIVIDEND ETFX 10.00 USD"));
    assert!(result.cgt_content.contains("DIVIDEND ETFX 20.00 USD"));
}

#[test]
fn test_dividend_with_multiple_tax_withholdings() {
    // NRA tax withholding split across multiple entries
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
                "Description": "TAX ADJ",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$10.00"
            },
            {
                "Date": "07/15/2023",
                "Action": "NRA Withholding",
                "Symbol": "FOO",
                "Description": "TAX WITHHOLD",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$5.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // Tax should be combined: 10 + 5 = 15
    assert!(
        result
            .cgt_content
            .contains("DIVIDEND FOO 100.00 USD TAX 15.00 USD")
    );
}

#[test]
fn test_dividend_without_tax() {
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "07/15/2023",
                "Action": "Cash Dividend",
                "Symbol": "ETFX",
                "Description": "ETF DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$50.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    // No TAX clause when no withholding
    assert!(result.cgt_content.contains("DIVIDEND ETFX 50.00 USD"));
    assert!(!result.cgt_content.contains("TAX"));
}

// ===========================================
// Variable CSV Field Counts
// ===========================================

#[test]
fn test_csv_with_extra_fields() {
    // Real exports sometimes have extra trailing fields
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.50",
                "Fees & Comm": "$4.95",
                "Amount": "-$1259.95",
                "ExtraField": "extra"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("2023-04-25 BUY XYZZ"));
}

#[test]
fn test_csv_with_fewer_fields() {
    // Some rows might have fewer fields (trailing commas omitted)
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/25/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$125.50",
                "Fees & Comm": "$4.95",
                "Amount": "-$1259.95"
            },
            {
                "Date": "05/01/2023",
                "Action": "Wire Sent",
                "Symbol": "--",
                "Description": "WIRE OUT",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();
    assert!(result.cgt_content.contains("BUY XYZZ"));
    assert!(result.cgt_content.contains("# SKIPPED: Wire Sent"));
}

// ===========================================
// Typical Full Workflow Test
// ===========================================

#[test]
fn test_typical_monthly_activity() {
    // Simulate a typical month with RSU vest, sell-to-cover, regular trades, and dividends
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "04/01/2023",
                "Action": "Cash Dividend",
                "Symbol": "ETFX",
                "Description": "DIVIDEND",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$25.50"
            },
            {
                "Date": "04/15/2023",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "RSU VEST",
                "Quantity": "100",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            },
            {
                "Date": "04/15/2023",
                "Action": "Sell",
                "Symbol": "ACME",
                "Description": "SELL TO COVER",
                "Quantity": "30",
                "Price": "$55.00",
                "Fees & Comm": "$0.05",
                "Amount": "$1649.95"
            },
            {
                "Date": "04/20/2023",
                "Action": "Buy",
                "Symbol": "FOO",
                "Description": "FOO INC",
                "Quantity": "5",
                "Price": "$165.00",
                "Fees & Comm": "$4.95",
                "Amount": "-$829.95"
            },
            {
                "Date": "04/25/2023",
                "Action": "Sell",
                "Symbol": "FOO",
                "Description": "FOO INC",
                "Quantity": "5",
                "Price": "$170.00",
                "Fees & Comm": "$4.95",
                "Amount": "$845.05"
            },
            {
                "Date": "04/28/2023",
                "Action": "Credit Interest",
                "Symbol": "--",
                "Description": "INTEREST",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "$1.25"
            },
            {
                "Date": "04/30/2023",
                "Action": "Wire Sent",
                "Symbol": "--",
                "Description": "TRANSFER OUT",
                "Quantity": null,
                "Price": null,
                "Fees & Comm": null,
                "Amount": "-$500.00"
            }
        ]
    }"#;

    let awards = r#"{
        "Transactions": [
            {
                "Date": "04/15/2023",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$55.00"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // Check all CGT-relevant transactions are present
    assert!(result.cgt_content.contains("DIVIDEND ETFX 25.50 USD"));
    assert!(result.cgt_content.contains("BUY ACME 100 @ 55.00 USD"));
    assert!(result.cgt_content.contains("SELL ACME 30 @ 55.00 USD"));
    assert!(result.cgt_content.contains("BUY FOO 5 @ 165.00 USD"));
    assert!(result.cgt_content.contains("SELL FOO 5 @ 170.00 USD"));

    // Check non-CGT transactions are skipped
    assert!(result.warnings.is_empty());
    assert_eq!(result.skipped_count, 2);

    // Verify chronological order
    let div_pos = result.cgt_content.find("DIVIDEND ETFX").unwrap();
    let rsu_pos = result.cgt_content.find("BUY ACME 100").unwrap();
    let foo_buy_pos = result.cgt_content.find("BUY FOO").unwrap();
    let foo_sell_pos = result.cgt_content.find("SELL FOO").unwrap();

    assert!(div_pos < rsu_pos);
    assert!(rsu_pos < foo_buy_pos);
    assert!(foo_buy_pos < foo_sell_pos);
}

#[test]
fn test_multi_year_transactions() {
    // Transactions spanning multiple years
    let json = r#"{
        "BrokerageTransactions": [
            {
                "Date": "12/15/2022",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$90.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$904.00"
            },
            {
                "Date": "03/15/2023",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "5",
                "Price": "$100.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$504.00"
            },
            {
                "Date": "06/15/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "10",
                "Price": "$110.00",
                "Fees & Comm": "$4.00",
                "Amount": "$1096.00"
            },
            {
                "Date": "12/15/2023",
                "Action": "Sell",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "5",
                "Price": "$120.00",
                "Fees & Comm": "$4.00",
                "Amount": "$596.00"
            },
            {
                "Date": "01/15/2024",
                "Action": "Buy",
                "Symbol": "XYZZ",
                "Description": "XYZZ CORP",
                "Quantity": "15",
                "Price": "$115.00",
                "Fees & Comm": "$4.00",
                "Amount": "-$1729.00"
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: json.to_string(),
        awards_json: None,
    };

    let result = converter.convert(&input).unwrap();

    // Check all transactions present and sorted
    assert!(result.cgt_content.contains("2022-12-15 BUY XYZZ"));
    assert!(result.cgt_content.contains("2023-03-15 BUY XYZZ"));
    assert!(result.cgt_content.contains("2023-06-15 SELL XYZZ"));
    assert!(result.cgt_content.contains("2023-12-15 SELL XYZZ"));
    assert!(result.cgt_content.contains("2024-01-15 BUY XYZZ"));

    // Verify chronological order
    let positions: Vec<_> = [
        "2022-12-15",
        "2023-03-15",
        "2023-06-15",
        "2023-12-15",
        "2024-01-15",
    ]
    .iter()
    .map(|date| result.cgt_content.find(date).unwrap())
    .collect();

    for i in 1..positions.len() {
        assert!(
            positions[i - 1] < positions[i],
            "Transactions not in chronological order"
        );
    }
}

// ===========================================
// RSU Vest Date for CGT Tests
// ===========================================

#[test]
fn test_rsu_same_day_matching_with_vest_date() {
    // Scenario: RSU vests on 01/15, shares settle on 01/17 (T+2)
    // Employee sells some shares on 01/15 (vest date)
    // CGT Same Day rule should match sale with acquisition on vest date
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "01/17/2024",
                "Action": "Stock Plan Activity",
                "Symbol": "ACME",
                "Description": "RSU VEST",
                "Quantity": "100",
                "Price": null,
                "Fees & Comm": null,
                "Amount": null
            },
            {
                "Date": "01/15/2024",
                "Action": "Sell",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "30",
                "Price": "$55.00",
                "Fees & Comm": "$0.05",
                "Amount": "$1649.95"
            }
        ]
    }"#;

    // Award vest date is 01/15 (matches the sale date)
    let awards = r#"{
        "Transactions": [
            {
                "Date": "01/15/2024",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$55.00"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // BUY should use vest date (01/15), enabling Same Day matching with the sale
    assert!(
        result
            .cgt_content
            .contains("2024-01-15 BUY ACME 100 @ 55.00 USD"),
        "RSU acquisition should use vest date 2024-01-15, not settlement date. Output: {}",
        result.cgt_content
    );
    assert!(
        result
            .cgt_content
            .contains("2024-01-15 SELL ACME 30 @ 55.00 USD"),
        "Sale should be on 2024-01-15"
    );

    // Both BUY and SELL are now on same date, enabling Same Day matching for CGT
}

#[test]
fn test_rsu_bnb_window_with_vest_date() {
    // Scenario: Employee sells shares on 01/10
    // RSU vests on 02/05 (26 days after sale) - within B&B 30-day window
    // Shares settle on 02/07 (28 days after sale)
    // B&B should use vest date (26 days) not settlement date (28 days)
    let transactions = r#"{
        "BrokerageTransactions": [
            {
                "Date": "01/10/2024",
                "Action": "Sell",
                "Symbol": "ACME",
                "Description": "ACME CORP",
                "Quantity": "30",
                "Price": "$50.00",
                "Fees & Comm": "$0.05",
                "Amount": "$1499.95"
            },
            {
                "Date": "02/07/2024",
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

    // Award vest date is 02/05 (26 days after sale, within B&B window)
    let awards = r#"{
        "Transactions": [
            {
                "Date": "02/05/2024",
                "Symbol": "ACME",
                "TransactionDetails": [
                    {"Details": {"FairMarketValuePrice": "$52.00"}}
                ]
            }
        ]
    }"#;

    let converter = SchwabConverter::new();
    let input = SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    };

    let result = converter.convert(&input).unwrap();

    // BUY should use vest date (02/05) which is 26 days after sale (within B&B window)
    // If settlement date (02/07 = 28 days) was used, it would still be within window,
    // but the cost basis date would be wrong
    assert!(
        result
            .cgt_content
            .contains("2024-02-05 BUY ACME 100 @ 52.00 USD"),
        "RSU acquisition should use vest date 2024-02-05. Output: {}",
        result.cgt_content
    );
}
