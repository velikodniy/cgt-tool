use crate::error::CgtError;
use crate::models::{Operation, Transaction};
use chrono::NaiveDate;
use pest::Parser;
use pest_derive::Parser;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct CgtParser;

pub fn parse_file(input: &str) -> Result<Vec<Transaction>, CgtError> {
    let pairs = CgtParser::parse(Rule::transaction_list, input).map_err(Box::new)?;
    let mut transactions = Vec::new();

    for pair in pairs {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::transaction {
                transactions.push(parse_transaction(inner)?);
            }
        }
    }

    Ok(transactions)
}

fn parse_transaction(pair: pest::iterators::Pair<Rule>) -> Result<Transaction, CgtError> {
    let mut inner = pair.into_inner();
    let date_pair = inner.next().unwrap();
    let date = NaiveDate::parse_from_str(date_pair.as_str(), "%Y-%m-%d")
        .map_err(|_| CgtError::InvalidTransaction("Invalid date format".to_string()))?;

    let command_pair = inner.next().unwrap();
    let command_inner = command_pair.into_inner().next().unwrap();

    let (ticker, operation) = match command_inner.as_rule() {
        Rule::cmd_buy => parse_buy_sell(command_inner, "BUY")?,
        Rule::cmd_sell => parse_buy_sell(command_inner, "SELL")?,
        Rule::cmd_dividend => parse_dividend(command_inner)?,
        Rule::cmd_capreturn => parse_capreturn(command_inner)?,
        Rule::cmd_split => parse_split(command_inner, "SPLIT")?,
        Rule::cmd_unsplit => parse_split(command_inner, "UNSPLIT")?,
        _ => return Err(CgtError::InvalidTransaction("Unknown command".to_string())),
    };

    Ok(Transaction {
        date,
        ticker,
        operation,
    })
}

fn parse_decimal(s: &str) -> Result<Decimal, CgtError> {
    Decimal::from_str(s)
        .map_err(|_| CgtError::InvalidTransaction(format!("Invalid decimal: {}", s)))
}

fn parse_buy_sell(
    pair: pest::iterators::Pair<Rule>,
    action: &str,
) -> Result<(String, Operation), CgtError> {
    // pair is cmd_buy or cmd_sell
    // inner is buy_sell_args
    let args = pair.into_inner().next().unwrap();
    let mut inner = args.into_inner();

    let ticker = inner.next().unwrap().as_str().to_string();
    let amount = parse_decimal(inner.next().unwrap().as_str())?;
    let price = parse_decimal(inner.next().unwrap().as_str())?;
    let expenses = parse_decimal(inner.next().unwrap().as_str())?;

    let op = match action {
        "BUY" => Operation::Buy {
            amount,
            price,
            expenses,
        },
        "SELL" => Operation::Sell {
            amount,
            price,
            expenses,
        },
        _ => unreachable!(),
    };
    Ok((ticker, op))
}

fn parse_dividend(pair: pest::iterators::Pair<Rule>) -> Result<(String, Operation), CgtError> {
    let args = pair.into_inner().next().unwrap();
    let mut inner = args.into_inner();

    let ticker = inner.next().unwrap().as_str().to_string();
    let amount = parse_decimal(inner.next().unwrap().as_str())?;
    let tax_paid = parse_decimal(inner.next().unwrap().as_str())?;

    Ok((ticker, Operation::Dividend { amount, tax_paid }))
}

fn parse_capreturn(pair: pest::iterators::Pair<Rule>) -> Result<(String, Operation), CgtError> {
    let args = pair.into_inner().next().unwrap();
    let mut inner = args.into_inner();

    let ticker = inner.next().unwrap().as_str().to_string();
    let amount = parse_decimal(inner.next().unwrap().as_str())?;
    let expenses = parse_decimal(inner.next().unwrap().as_str())?;

    Ok((ticker, Operation::CapReturn { amount, expenses }))
}

fn parse_split(
    pair: pest::iterators::Pair<Rule>,
    action: &str,
) -> Result<(String, Operation), CgtError> {
    let args = pair.into_inner().next().unwrap();
    let mut inner = args.into_inner();

    let ticker = inner.next().unwrap().as_str().to_string();
    let ratio = parse_decimal(inner.next().unwrap().as_str())?;

    let op = match action {
        "SPLIT" => Operation::Split { ratio },
        "UNSPLIT" => Operation::Unsplit { ratio },
        _ => unreachable!(),
    };
    Ok((ticker, op))
}
