//! DSL serializer for transactions.
//! The inverse of the parser - converts Transaction structs back to DSL text.

use crate::model::{Operation, Transaction};
use crate::money::CurrencyAmount;

/// Convert a single transaction to DSL format.
pub fn serialize_transaction(tx: &Transaction) -> String {
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
            if !is_default(fees) {
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
            if !is_default(fees) {
                line.push_str(&format!(" FEES {}", format_amount(fees)));
            }
            line
        }
        Operation::Dividend {
            total_value,
            tax_paid,
        } => {
            let mut line = format!(
                "{} DIVIDEND {} TOTAL {}",
                date,
                tx.ticker,
                format_amount(total_value)
            );
            if !is_default(tax_paid) {
                line.push_str(&format!(" TAX {}", format_amount(tax_paid)));
            }
            line
        }
        Operation::Accumulation {
            amount,
            total_value,
            tax_paid,
        } => {
            let mut line = format!(
                "{} ACCUMULATION {} {} TOTAL {}",
                date,
                tx.ticker,
                amount,
                format_amount(total_value)
            );
            if !is_default(tax_paid) {
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
            if !is_default(fees) {
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
pub fn serialize(transactions: &[Transaction]) -> String {
    transactions
        .iter()
        .map(serialize_transaction)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a `CurrencyAmount` for DSL output (e.g. `150 USD` or `120 GBP`).
fn format_amount(amount: &CurrencyAmount) -> String {
    format!("{} {}", amount.amount, amount.code())
}

/// True when an optional amount equals the parser's default for an absent
/// field (zero GBP), so omitting it loses no information. A zero amount in
/// any other currency must still be written to keep roundtrips lossless.
fn is_default(amount: &CurrencyAmount) -> bool {
    amount.amount.is_zero() && amount.is_gbp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::parse;
    use crate::money::Currency;
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
        assert_eq!(
            serialize_transaction(&t),
            "2024-01-15 BUY VOD 100 @ 120 GBP"
        );
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
            serialize_transaction(&t),
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
            serialize_transaction(&t),
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
        assert_eq!(
            serialize_transaction(&t),
            "2024-06-20 SELL VOD 50 @ 130 GBP"
        );
    }

    #[test]
    fn dividend_with_tax() {
        let t = tx(
            (2024, 3, 1),
            "VWRL",
            Operation::Dividend {
                total_value: gbp(dec!(50)),
                tax_paid: gbp(dec!(5)),
            },
        );
        assert_eq!(
            serialize_transaction(&t),
            "2024-03-01 DIVIDEND VWRL TOTAL 50 GBP TAX 5 GBP"
        );
    }

    #[test]
    fn zero_foreign_currency_fees_kept() {
        // A zero fee in a non-GBP currency is not the parser's default for an
        // absent FEES clause, so dropping it would make roundtrips lossy.
        let t = tx(
            (2020, 5, 15),
            "ACME",
            Operation::Buy {
                amount: dec!(500),
                price: usd(dec!(25.00)),
                fees: usd(dec!(0)),
            },
        );
        assert_eq!(
            serialize_transaction(&t),
            "2020-05-15 BUY ACME 500 @ 25.00 USD FEES 0 USD"
        );
    }

    #[test]
    fn dividend_zero_tax_omitted() {
        let t = tx(
            (2024, 3, 1),
            "VWRL",
            Operation::Dividend {
                total_value: gbp(dec!(50)),
                tax_paid: gbp(dec!(0)),
            },
        );
        let dsl = serialize_transaction(&t);
        assert_eq!(dsl, "2024-03-01 DIVIDEND VWRL TOTAL 50 GBP");
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
            serialize_transaction(&t),
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
            serialize_transaction(&t),
            "2024-05-10 CAPRETURN BHP 200 TOTAL 100 GBP"
        );
    }

    #[test]
    fn split() {
        let t = tx((2024, 6, 1), "NVDA", Operation::Split { ratio: dec!(4) });
        assert_eq!(serialize_transaction(&t), "2024-06-01 SPLIT NVDA RATIO 4");
    }

    #[test]
    fn unsplit() {
        let t = tx((2024, 7, 1), "TEST", Operation::Unsplit { ratio: dec!(2) });
        assert_eq!(serialize_transaction(&t), "2024-07-01 UNSPLIT TEST RATIO 2");
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
        let dsl = serialize(&txs);
        assert_eq!(
            dsl,
            "2024-01-15 BUY AAPL 100 @ 150 GBP\n2024-06-20 SELL AAPL 50 @ 180 GBP"
        );
    }

    #[test]
    fn empty_transactions() {
        assert_eq!(serialize(&[]), "");
    }

    #[test]
    fn roundtrip_buy_sell() {
        let input = "2024-01-15 BUY AAPL 100 @ 150 GBP FEES 10 GBP\n\
                      2024-06-20 SELL AAPL 50 @ 180 GBP FEES 5 GBP";
        let parsed = parse(input).expect("parse should succeed");
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_dividend() {
        let input = "2024-03-01 DIVIDEND VWRL TOTAL 50 GBP TAX 5 GBP";
        let parsed = parse(input).expect("parse should succeed");
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_split_unsplit() {
        let input = "2024-06-01 SPLIT NVDA RATIO 4\n\
                      2024-07-01 UNSPLIT TEST RATIO 2";
        let parsed = parse(input).expect("parse should succeed");
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_capreturn() {
        let input = "2024-05-10 CAPRETURN BHP 200 TOTAL 100 GBP FEES 2 GBP";
        let parsed = parse(input).expect("parse should succeed");
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_foreign_currency() {
        let input = "2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD";
        let parsed = parse(input).expect("parse should succeed");
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).expect("reparse should succeed");
        assert_eq!(parsed, reparsed);
    }
}
