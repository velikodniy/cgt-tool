use crate::error::CgtError;
use crate::models::{CurrencyAmount, Operation, Transaction};
use cgt_money::Currency;
use chrono::NaiveDate;
use pest_consume::{Error, Parser, match_nodes};
use rust_decimal::Decimal;
use std::str::FromStr;

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

// Generate the pest parser from grammar
#[derive(Parser)]
#[grammar = "parser.pest"]
struct CgtParser;

#[pest_consume::parser]
impl CgtParser {
    fn EOI(_input: Node) -> ParseResult<()> {
        Ok(())
    }

    fn COMMENT(_input: Node) -> ParseResult<()> {
        Ok(())
    }

    fn date(input: Node) -> ParseResult<NaiveDate> {
        NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d")
            .map_err(|_| input.error("Invalid date format"))
    }

    fn ticker(input: Node) -> ParseResult<String> {
        Ok(input.as_str().to_uppercase())
    }

    fn quantity(input: Node) -> ParseResult<Decimal> {
        parse_decimal_str(input.as_str(), &input)
    }

    fn ratio(input: Node) -> ParseResult<Decimal> {
        parse_decimal_str(input.as_str(), &input)
    }

    fn decimal(input: Node) -> ParseResult<Decimal> {
        parse_decimal_str(input.as_str(), &input)
    }

    fn currency_code(input: Node) -> ParseResult<Currency> {
        let code = input.as_str().to_uppercase();
        Currency::from_code(&code)
            .ok_or_else(|| input.error(format!("Invalid currency code: {code}")))
    }

    fn money(input: Node) -> ParseResult<CurrencyAmount> {
        Ok(match_nodes!(input.into_children();
            [decimal(amount)] => CurrencyAmount::new(amount, Currency::GBP),
            [decimal(amount), currency_code(currency)] => CurrencyAmount::new(amount, currency),
        ))
    }

    fn price(input: Node) -> ParseResult<CurrencyAmount> {
        Ok(match_nodes!(input.into_children();
            [money(m)] => m,
        ))
    }

    fn total_value(input: Node) -> ParseResult<CurrencyAmount> {
        Ok(match_nodes!(input.into_children();
            [money(m)] => m,
        ))
    }

    fn fees(input: Node) -> ParseResult<CurrencyAmount> {
        Ok(match_nodes!(input.into_children();
            [money(m)] => m,
        ))
    }

    fn tax(input: Node) -> ParseResult<CurrencyAmount> {
        Ok(match_nodes!(input.into_children();
            [money(m)] => m,
        ))
    }

    fn ratio_value(input: Node) -> ParseResult<Decimal> {
        Ok(match_nodes!(input.into_children();
            [ratio(r)] => r,
        ))
    }

    fn cmd_buy(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), quantity(q), price(p)] => {
                (t, Operation::Buy {
                    amount: q,
                    price: p,
                    fees: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
                })
            },
            [ticker(t), quantity(q), price(p), fees(f)] => {
                (t, Operation::Buy {
                    amount: q,
                    price: p,
                    fees: f,
                })
            },
        ))
    }

    fn cmd_sell(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), quantity(q), price(p)] => {
                (t, Operation::Sell {
                    amount: q,
                    price: p,
                    fees: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
                })
            },
            [ticker(t), quantity(q), price(p), fees(f)] => {
                (t, Operation::Sell {
                    amount: q,
                    price: p,
                    fees: f,
                })
            },
        ))
    }

    fn cmd_dividend(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), quantity(q), total_value(tv)] => {
                (t, Operation::Dividend {
                    amount: q,
                    total_value: tv,
                    tax_paid: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
                })
            },
            [ticker(t), quantity(q), total_value(tv), tax(tx)] => {
                (t, Operation::Dividend {
                    amount: q,
                    total_value: tv,
                    tax_paid: tx,
                })
            },
        ))
    }

    fn cmd_accumulation(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), quantity(q), total_value(tv)] => {
                (t, Operation::Accumulation {
                    amount: q,
                    total_value: tv,
                    tax_paid: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
                })
            },
            [ticker(t), quantity(q), total_value(tv), tax(tx)] => {
                (t, Operation::Accumulation {
                    amount: q,
                    total_value: tv,
                    tax_paid: tx,
                })
            },
        ))
    }

    fn cmd_capreturn(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), quantity(q), total_value(tv)] => {
                (t, Operation::CapReturn {
                    amount: q,
                    total_value: tv,
                    fees: CurrencyAmount::new(Decimal::ZERO, Currency::GBP),
                })
            },
            [ticker(t), quantity(q), total_value(tv), fees(f)] => {
                (t, Operation::CapReturn {
                    amount: q,
                    total_value: tv,
                    fees: f,
                })
            },
        ))
    }

    fn cmd_split(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), ratio_value(r)] => {
                (t, Operation::Split { ratio: r })
            },
        ))
    }

    fn cmd_unsplit(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [ticker(t), ratio_value(r)] => {
                (t, Operation::Unsplit { ratio: r })
            },
        ))
    }

    fn command(input: Node) -> ParseResult<(String, Operation<CurrencyAmount>)> {
        Ok(match_nodes!(input.into_children();
            [cmd_buy(c)] => c,
            [cmd_sell(c)] => c,
            [cmd_dividend(c)] => c,
            [cmd_accumulation(c)] => c,
            [cmd_capreturn(c)] => c,
            [cmd_split(c)] => c,
            [cmd_unsplit(c)] => c,
        ))
    }

    fn transaction(input: Node) -> ParseResult<Transaction> {
        Ok(match_nodes!(input.into_children();
            [date(d), command((ticker, operation))] => {
                Transaction {
                    date: d,
                    ticker,
                    operation,
                }
            },
        ))
    }

    fn transaction_list(input: Node) -> ParseResult<Vec<Transaction>> {
        let mut transactions = Vec::new();
        for child in input.into_children() {
            match child.as_rule() {
                Rule::transaction => transactions.push(Self::transaction(child)?),
                Rule::COMMENT | Rule::EOI => {}
                _ => {}
            }
        }
        Ok(transactions)
    }
}

// Helper function to parse decimal strings
fn parse_decimal_str(s: &str, node: &Node) -> ParseResult<Decimal> {
    Decimal::from_str(s).map_err(|_| node.error(format!("Invalid decimal: {s}")))
}

/// Parse a CGT file. Amounts are parsed with their original currency; GBP conversion
/// is deferred to calculation time.
pub fn parse_file(input: &str) -> std::result::Result<Vec<Transaction>, CgtError> {
    let inputs = CgtParser::parse(Rule::transaction_list, input)
        .map_err(|e| CgtError::ParseError(Box::new(e)))?;

    let input = inputs
        .single()
        .map_err(|e| CgtError::ParseError(Box::new(e)))?;

    CgtParser::transaction_list(input).map_err(Into::into)
}
