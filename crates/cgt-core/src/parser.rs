use crate::error::{CgtError, ParseErrorContext, suggest_transaction_type};
use crate::models::{Operation, Transaction};
use chrono::NaiveDate;
use pest::Parser;
use pest::error::LineColLocation;
use pest_derive::Parser;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct CgtParser;

/// Convert a pest error to a rich ParseErrorContext.
fn from_pest_error(err: &pest::error::Error<Rule>, input: &str) -> ParseErrorContext {
    let (line, column) = match err.line_col {
        LineColLocation::Pos((l, c)) => (l, c),
        LineColLocation::Span((l, c), _) => (l, c),
    };

    // Get the line content
    let line_content = input
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .to_string();

    // Extract what was found at the error location
    let found = extract_found_token(&line_content, column);

    // Format expected rules nicely
    let expected = format_expected_rules(err);

    // Try to suggest corrections for transaction types
    let suggestion = if found.len() <= 12 {
        suggest_transaction_type(&found).map(|s| format!("Did you mean '{}'?", s))
    } else {
        None
    };

    let mut ctx = ParseErrorContext::new(line, column, found, expected, line_content);
    if let Some(s) = suggestion {
        ctx = ctx.with_suggestion(s);
    }
    ctx
}

/// Extract the token found at the error location.
fn extract_found_token(line: &str, column: usize) -> String {
    let start = column.saturating_sub(1);
    if start >= line.len() {
        return "end of line".to_string();
    }

    let chars: Vec<char> = line.chars().collect();
    let mut end = start;

    // Extend to end of word
    while end < chars.len() && !chars[end].is_whitespace() {
        end += 1;
    }

    if start == end {
        "unexpected character".to_string()
    } else {
        chars[start..end].iter().collect()
    }
}

/// Format the expected rules from a pest error into a human-readable string.
fn format_expected_rules(err: &pest::error::Error<Rule>) -> String {
    match &err.variant {
        pest::error::ErrorVariant::ParsingError { positives, .. } => {
            let rules: Vec<String> = positives.iter().map(|r| format_rule_name(*r)).collect();
            if rules.is_empty() {
                "valid input".to_string()
            } else if rules.len() == 1 {
                rules[0].clone()
            } else {
                format!("one of: {}", rules.join(", "))
            }
        }
        pest::error::ErrorVariant::CustomError { message } => message.clone(),
    }
}

/// Convert a Rule to a human-readable name.
fn format_rule_name(rule: Rule) -> String {
    match rule {
        Rule::transaction => "transaction".to_string(),
        Rule::transaction_list => "transaction list".to_string(),
        Rule::date => "date (YYYY-MM-DD)".to_string(),
        Rule::command => "command (BUY, SELL, DIVIDEND, etc.)".to_string(),
        Rule::cmd_buy => "BUY command".to_string(),
        Rule::cmd_sell => "SELL command".to_string(),
        Rule::cmd_dividend => "DIVIDEND command".to_string(),
        Rule::cmd_capreturn => "CAPRETURN command".to_string(),
        Rule::cmd_split => "SPLIT command".to_string(),
        Rule::cmd_unsplit => "UNSPLIT command".to_string(),
        Rule::ticker => "ticker symbol".to_string(),
        Rule::amount => "amount (number)".to_string(),
        Rule::price => "price (number)".to_string(),
        Rule::decimal => "decimal number".to_string(),
        Rule::total_value => "total value".to_string(),
        Rule::expenses => "expenses amount".to_string(),
        Rule::tax_paid => "tax paid amount".to_string(),
        Rule::ratio => "split ratio".to_string(),
        Rule::buy_sell_args => "BUY/SELL arguments (ticker amount @ price)".to_string(),
        Rule::dividend_args => "DIVIDEND arguments".to_string(),
        Rule::capreturn_args => "CAPRETURN arguments".to_string(),
        Rule::split_args => "SPLIT arguments (ticker RATIO value)".to_string(),
        Rule::EOI => "end of input".to_string(),
        _ => format!("{:?}", rule),
    }
}

pub fn parse_file(input: &str) -> Result<Vec<Transaction>, CgtError> {
    let pairs = CgtParser::parse(Rule::transaction_list, input).map_err(|err| {
        let ctx = from_pest_error(&err, input);
        CgtError::ParseErrorContext(ctx)
    })?;
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
    let date_pair = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "date" })?;
    let date = NaiveDate::parse_from_str(date_pair.as_str(), "%Y-%m-%d")
        .map_err(|_| CgtError::InvalidTransaction("Invalid date format".to_string()))?;

    let command_pair = inner.next().ok_or(CgtError::UnexpectedParserState {
        expected: "command",
    })?;
    let command_inner =
        command_pair
            .into_inner()
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "command inner",
            })?;

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
    let args = pair
        .into_inner()
        .next()
        .ok_or(CgtError::UnexpectedParserState {
            expected: "buy/sell arguments",
        })?;
    let mut inner = args.into_inner();

    let ticker = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "ticker" })?
        .as_str()
        .to_uppercase();
    let amount = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState { expected: "amount" })?
            .as_str(),
    )?;
    let price = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState { expected: "price" })?
            .as_str(),
    )?;

    let expenses = if let Some(expenses_value_pair) = inner.next() {
        parse_decimal(expenses_value_pair.as_str())?
    } else {
        Decimal::ZERO
    };

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
        _ => {
            return Err(CgtError::UnexpectedParserState {
                expected: "BUY or SELL action",
            });
        }
    };
    Ok((ticker, op))
}

fn parse_dividend(pair: pest::iterators::Pair<Rule>) -> Result<(String, Operation), CgtError> {
    let args = pair
        .into_inner()
        .next()
        .ok_or(CgtError::UnexpectedParserState {
            expected: "dividend arguments",
        })?;
    let mut inner = args.into_inner();

    let ticker = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "ticker" })?
        .as_str()
        .to_uppercase();
    let amount = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState { expected: "amount" })?
            .as_str(),
    )?;
    let total_value = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "total value",
            })?
            .as_str(),
    )?;
    let tax_paid = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "tax paid",
            })?
            .as_str(),
    )?;

    Ok((
        ticker,
        Operation::Dividend {
            amount,
            total_value,
            tax_paid,
        },
    ))
}

fn parse_capreturn(pair: pest::iterators::Pair<Rule>) -> Result<(String, Operation), CgtError> {
    let args = pair
        .into_inner()
        .next()
        .ok_or(CgtError::UnexpectedParserState {
            expected: "capreturn arguments",
        })?;
    let mut inner = args.into_inner();

    let ticker = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "ticker" })?
        .as_str()
        .to_uppercase();
    let amount = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState { expected: "amount" })?
            .as_str(),
    )?;
    let total_value = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "total value",
            })?
            .as_str(),
    )?;
    let expenses = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "expenses",
            })?
            .as_str(),
    )?;

    Ok((
        ticker,
        Operation::CapReturn {
            amount,
            total_value,
            expenses,
        },
    ))
}

fn parse_split(
    pair: pest::iterators::Pair<Rule>,
    action: &str,
) -> Result<(String, Operation), CgtError> {
    let args = pair
        .into_inner()
        .next()
        .ok_or(CgtError::UnexpectedParserState {
            expected: "split arguments",
        })?;
    let mut inner = args.into_inner();

    let ticker = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "ticker" })?
        .as_str()
        .to_uppercase();
    let ratio = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState { expected: "ratio" })?
            .as_str(),
    )?;

    let op = match action {
        "SPLIT" => Operation::Split { ratio },
        "UNSPLIT" => Operation::Unsplit { ratio },
        _ => {
            return Err(CgtError::UnexpectedParserState {
                expected: "SPLIT or UNSPLIT action",
            });
        }
    };
    Ok((ticker, op))
}
