//! DSL serializer for transactions.
//! The inverse of the parser - converts Transaction structs back to DSL text.

use crate::models::{Operation, Transaction};
use cgt_money::CurrencyAmount;

/// Convert a single transaction to DSL format.
pub fn transaction_to_dsl(tx: &Transaction) -> String {
    let date = tx.date.format("%Y-%m-%d");

    match &tx.operation {
        Operation::Buy {
            amount,
            price,
            fees,
        } => {
            let mut line = format!(
                "{} BUY {} {} @ {}",
                date,
                tx.ticker,
                amount,
                format_amount(price)
            );
            if !fees.amount.is_zero() {
                line.push_str(&format!(" FEES {}", format_amount(fees)));
            }
            line
        }
        Operation::Sell {
            amount,
            price,
            fees,
        } => {
            let mut line = format!(
                "{} SELL {} {} @ {}",
                date,
                tx.ticker,
                amount,
                format_amount(price)
            );
            if !fees.amount.is_zero() {
                line.push_str(&format!(" FEES {}", format_amount(fees)));
            }
            line
        }
        Operation::Dividend {
            amount,
            total_value,
            tax_paid,
        } => {
            let mut line = format!(
                "{} DIVIDEND {} {} TOTAL {}",
                date,
                tx.ticker,
                amount,
                format_amount(total_value)
            );
            if !tax_paid.amount.is_zero() {
                line.push_str(&format!(" TAX {}", format_amount(tax_paid)));
            }
            line
        }
        Operation::CapReturn {
            amount,
            total_value,
            fees,
        } => {
            let mut line = format!(
                "{} CAPRETURN {} {} TOTAL {}",
                date,
                tx.ticker,
                amount,
                format_amount(total_value)
            );
            if !fees.amount.is_zero() {
                line.push_str(&format!(" FEES {}", format_amount(fees)));
            }
            line
        }
        Operation::Split { ratio } => {
            format!("{} SPLIT {} RATIO {}", date, tx.ticker, ratio)
        }
        Operation::Unsplit { ratio } => {
            format!("{} UNSPLIT {} RATIO {}", date, tx.ticker, ratio)
        }
    }
}

/// Convert a slice of transactions to DSL format, one line per transaction.
pub fn transactions_to_dsl(transactions: &[Transaction]) -> String {
    transactions
        .iter()
        .map(transaction_to_dsl)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a `CurrencyAmount` for DSL output (e.g. `150 USD` or `120 GBP`).
fn format_amount(amount: &CurrencyAmount) -> String {
    format!("{} {}", amount.amount, amount.code())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_file;
    use cgt_money::Currency;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn gbp(val: rust_decimal::Decimal) -> CurrencyAmount {
        CurrencyAmount::new(val, Currency::GBP)
    }

    fn usd(val: rust_decimal::Decimal) -> CurrencyAmount {
        CurrencyAmount::new(val, Currency::USD)
    }

    fn tx(date: (i32, u32, u32), ticker: &str, op: Operation<CurrencyAmount>) -> Transaction {
        Transaction {
            date: NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap(),
            ticker: ticker.to_string(),
            operation: op,
        }
    }

    #[test]
    fn buy_gbp_no_fees() {
        let t = tx(
            (2024, 1, 15),
            "VOD",
            Operation::Buy {
                amount: dec!(100),
                price: gbp(dec!(120)),
                fees: gbp(dec!(0)),
            },
        );
        assert_eq!(transaction_to_dsl(&t), "2024-01-15 BUY VOD 100 @ 120 GBP");
    }

    #[test]
    fn buy_usd_with_fees() {
        let t = tx(
            (2024, 1, 15),
            "AAPL",
            Operation::Buy {
                amount: dec!(100),
                price: usd(dec!(150)),
                fees: usd(dec!(10)),
            },
        );
        assert_eq!(
            transaction_to_dsl(&t),
            "2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD"
        );
    }

    #[test]
    fn sell_with_fees() {
        let t = tx(
            (2024, 6, 20),
            "AAPL",
            Operation::Sell {
                amount: dec!(50),
                price: gbp(dec!(180)),
                fees: gbp(dec!(5)),
            },
        );
        assert_eq!(
            transaction_to_dsl(&t),
            "2024-06-20 SELL AAPL 50 @ 180 GBP FEES 5 GBP"
        );
    }

    #[test]
    fn sell_no_fees() {
        let t = tx(
            (2024, 6, 20),
            "VOD",
            Operation::Sell {
                amount: dec!(50),
                price: gbp(dec!(130)),
                fees: gbp(dec!(0)),
            },
        );
        assert_eq!(transaction_to_dsl(&t), "2024-06-20 SELL VOD 50 @ 130 GBP");
    }

    #[test]
    fn dividend_with_tax() {
        let t = tx(
            (2024, 3, 1),
            "VWRL",
            Operation::Dividend {
                amount: dec!(100),
                total_value: gbp(dec!(50)),
                tax_paid: gbp(dec!(5)),
            },
        );
        assert_eq!(
            transaction_to_dsl(&t),
            "2024-03-01 DIVIDEND VWRL 100 TOTAL 50 GBP TAX 5 GBP"
        );
    }

    #[test]
    fn dividend_zero_tax_omitted() {
        let t = tx(
            (2024, 3, 1),
            "VWRL",
            Operation::Dividend {
                amount: dec!(100),
                total_value: gbp(dec!(50)),
                tax_paid: gbp(dec!(0)),
            },
        );
        let dsl = transaction_to_dsl(&t);
        assert_eq!(dsl, "2024-03-01 DIVIDEND VWRL 100 TOTAL 50 GBP");
        assert!(!dsl.contains("TAX"));
    }

    #[test]
    fn capreturn_with_fees() {
        let t = tx(
            (2024, 5, 10),
            "BHP",
            Operation::CapReturn {
                amount: dec!(200),
                total_value: gbp(dec!(100)),
                fees: gbp(dec!(2)),
            },
        );
        assert_eq!(
            transaction_to_dsl(&t),
            "2024-05-10 CAPRETURN BHP 200 TOTAL 100 GBP FEES 2 GBP"
        );
    }

    #[test]
    fn capreturn_no_fees() {
        let t = tx(
            (2024, 5, 10),
            "BHP",
            Operation::CapReturn {
                amount: dec!(200),
                total_value: gbp(dec!(100)),
                fees: gbp(dec!(0)),
            },
        );
        assert_eq!(
            transaction_to_dsl(&t),
            "2024-05-10 CAPRETURN BHP 200 TOTAL 100 GBP"
        );
    }

    #[test]
    fn split() {
        let t = tx((2024, 6, 1), "NVDA", Operation::Split { ratio: dec!(4) });
        assert_eq!(transaction_to_dsl(&t), "2024-06-01 SPLIT NVDA RATIO 4");
    }

    #[test]
    fn unsplit() {
        let t = tx((2024, 7, 1), "TEST", Operation::Unsplit { ratio: dec!(2) });
        assert_eq!(transaction_to_dsl(&t), "2024-07-01 UNSPLIT TEST RATIO 2");
    }

    #[test]
    fn multiple_transactions() {
        let txs = vec![
            tx(
                (2024, 1, 15),
                "AAPL",
                Operation::Buy {
                    amount: dec!(100),
                    price: gbp(dec!(150)),
                    fees: gbp(dec!(0)),
                },
            ),
            tx(
                (2024, 6, 20),
                "AAPL",
                Operation::Sell {
                    amount: dec!(50),
                    price: gbp(dec!(180)),
                    fees: gbp(dec!(0)),
                },
            ),
        ];
        let dsl = transactions_to_dsl(&txs);
        assert_eq!(
            dsl,
            "2024-01-15 BUY AAPL 100 @ 150 GBP\n2024-06-20 SELL AAPL 50 @ 180 GBP"
        );
    }

    #[test]
    fn empty_transactions() {
        assert_eq!(transactions_to_dsl(&[]), "");
    }

    #[test]
    fn roundtrip_buy_sell() {
        let input = "2024-01-15 BUY AAPL 100 @ 150 GBP FEES 10 GBP\n\
                      2024-06-20 SELL AAPL 50 @ 180 GBP FEES 5 GBP";
        let parsed = parse_file(input).expect("parse should succeed");
        let serialized = transactions_to_dsl(&parsed);
        let reparsed = parse_file(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_dividend() {
        let input = "2024-03-01 DIVIDEND VWRL 100 TOTAL 50 GBP TAX 5 GBP";
        let parsed = parse_file(input).expect("parse should succeed");
        let serialized = transactions_to_dsl(&parsed);
        let reparsed = parse_file(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_split_unsplit() {
        let input = "2024-06-01 SPLIT NVDA RATIO 4\n\
                      2024-07-01 UNSPLIT TEST RATIO 2";
        let parsed = parse_file(input).expect("parse should succeed");
        let serialized = transactions_to_dsl(&parsed);
        let reparsed = parse_file(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_capreturn() {
        let input = "2024-05-10 CAPRETURN BHP 200 TOTAL 100 GBP FEES 2 GBP";
        let parsed = parse_file(input).expect("parse should succeed");
        let serialized = transactions_to_dsl(&parsed);
        let reparsed = parse_file(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_foreign_currency() {
        let input = "2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD";
        let parsed = parse_file(input).expect("parse should succeed");
        let serialized = transactions_to_dsl(&parsed);
        let reparsed = parse_file(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }
}
