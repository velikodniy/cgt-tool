use cgt::model::{Currency, CurrencyAmount, Operation, Transaction};
use rust_decimal::Decimal;

fn gbp(amount: i64) -> CurrencyAmount {
    CurrencyAmount::new(Decimal::from(amount), Currency::GBP)
}

#[test]
fn json_and_direct_inputs_share_semantic_errors() -> Result<(), Box<dyn std::error::Error>> {
    let operations = [
        Operation::Buy {
            amount: 0.into(),
            price: gbp(10),
            fees: gbp(0),
        },
        Operation::Buy {
            amount: (-1).into(),
            price: gbp(10),
            fees: gbp(0),
        },
        Operation::Buy {
            amount: 1.into(),
            price: gbp(-1),
            fees: gbp(0),
        },
        Operation::Buy {
            amount: 1.into(),
            price: gbp(10),
            fees: gbp(-1),
        },
        Operation::Sell {
            amount: 0.into(),
            price: gbp(10),
            fees: gbp(0),
        },
        Operation::Sell {
            amount: 1.into(),
            price: gbp(-1),
            fees: gbp(0),
        },
        Operation::Sell {
            amount: 1.into(),
            price: gbp(10),
            fees: gbp(-1),
        },
        Operation::Dividend {
            total_value: gbp(-1),
            tax_paid: gbp(0),
        },
        Operation::Dividend {
            total_value: gbp(10),
            tax_paid: gbp(-1),
        },
        Operation::Accumulation {
            amount: (-1).into(),
            total_value: gbp(10),
            tax_paid: gbp(0),
        },
        Operation::Accumulation {
            amount: 1.into(),
            total_value: gbp(-1),
            tax_paid: gbp(0),
        },
        Operation::Accumulation {
            amount: 1.into(),
            total_value: gbp(10),
            tax_paid: gbp(-1),
        },
        Operation::CapReturn {
            amount: 0.into(),
            total_value: gbp(10),
            fees: gbp(0),
        },
        Operation::CapReturn {
            amount: 1.into(),
            total_value: gbp(-1),
            fees: gbp(0),
        },
        Operation::CapReturn {
            amount: 1.into(),
            total_value: gbp(10),
            fees: gbp(-1),
        },
        Operation::Split {
            ratio: Decimal::ZERO,
        },
        Operation::Split { ratio: (-1).into() },
        Operation::Unsplit {
            ratio: Decimal::ZERO,
        },
        Operation::Unsplit { ratio: (-1).into() },
        Operation::Unsplit {
            ratio: Decimal::new(15, 1),
        },
    ];
    for operation in operations {
        let transaction = Transaction {
            date: "2024-01-01".parse()?,
            ticker: "ABC".into(),
            operation,
            line: Some(42),
        };
        let validation = cgt::validate(std::slice::from_ref(&transaction));
        assert!(!validation.is_valid());
        let error = &validation.errors[0];
        assert_eq!(error.line, Some(42));
        assert_eq!(error.date, transaction.date);
        assert_eq!(error.ticker, transaction.ticker);
        let json_result =
            serde_json::from_value::<Transaction>(serde_json::to_value(&transaction)?);
        let Err(json_error) = json_result else {
            return Err("JSON accepted an invalid transaction".into());
        };
        assert!(json_error.to_string().contains(&error.message));
        assert!(json_error.to_string().contains("ABC on 2024-01-01"));
    }
    Ok(())
}

#[test]
fn zero_accumulation_and_free_acquisitions_remain_valid() -> Result<(), Box<dyn std::error::Error>>
{
    for operation in [
        Operation::Accumulation {
            amount: 0.into(),
            total_value: gbp(20),
            tax_paid: gbp(3),
        },
        Operation::Buy {
            amount: 1.into(),
            price: gbp(0),
            fees: gbp(0),
        },
        Operation::Dividend {
            total_value: gbp(0),
            tax_paid: gbp(0),
        },
        Operation::Split {
            ratio: Decimal::new(15, 1),
        },
    ] {
        let transaction = Transaction {
            date: "2024-01-01".parse()?,
            ticker: "BRK/B".into(),
            operation,
            line: None,
        };
        assert!(cgt::validate(std::slice::from_ref(&transaction)).is_valid());
        let decoded: Transaction = serde_json::from_value(serde_json::to_value(&transaction)?)?;
        assert_eq!(decoded, transaction);
    }
    Ok(())
}

#[test]
fn dsl_parse_retains_invalid_quantities_for_located_validation()
-> Result<(), Box<dyn std::error::Error>> {
    let transactions = cgt::dsl::parse("# Input header\n2024-01-01 BUY ABC 0 @ 10")?;
    let result = cgt::validate(&transactions);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].line, Some(2));
    assert_eq!(result.errors[0].message, "BUY with zero quantity");
    Ok(())
}

#[test]
fn invalid_tickers_are_rejected_for_json_and_direct_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    for ticker in ["", "ABC DEF", "ABC#", ".ABC"] {
        let transaction = Transaction {
            date: "2024-01-01".parse()?,
            ticker: ticker.into(),
            operation: Operation::Buy {
                amount: 1.into(),
                price: gbp(10),
                fees: gbp(0),
            },
            line: None,
        };
        assert!(!cgt::validate(std::slice::from_ref(&transaction)).is_valid());
        assert!(
            serde_json::from_value::<Transaction>(serde_json::to_value(&transaction)?).is_err()
        );
    }
    Ok(())
}
