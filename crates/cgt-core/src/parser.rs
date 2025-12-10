use crate::error::{CgtError, ParseErrorContext, suggest_transaction_type};
use crate::models::{CurrencyAmount, Operation, Transaction};
use cgt_fx::FxCache;
use chrono::{Datelike, NaiveDate};
use iso_currency::Currency;
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

    let line_content = input
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .to_string();

    let found = extract_found_token(&line_content, column);
    let expected = format_expected_rules(err);

    let suggestion = if found.len() <= 12 {
        suggest_transaction_type(&found).map(|s| format!("Did you mean '{s}'?"))
    } else {
        None
    };

    let mut ctx = ParseErrorContext::new(line, column, found, expected, line_content);
    if let Some(s) = suggestion {
        ctx = ctx.with_suggestion(s);
    }
    ctx
}

fn extract_found_token(line: &str, column: usize) -> String {
    let start = column.saturating_sub(1);
    if start >= line.len() {
        return "end of line".to_string();
    }

    let chars: Vec<char> = line.chars().collect();
    let mut end = start;

    while end < chars.len() && !chars[end].is_whitespace() {
        end += 1;
    }

    if start == end {
        "unexpected character".to_string()
    } else {
        chars[start..end].iter().collect()
    }
}

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
        Rule::quantity => "quantity (number)".to_string(),
        Rule::money => "amount with optional currency".to_string(),
        Rule::decimal => "decimal number".to_string(),
        Rule::ratio => "split ratio".to_string(),
        Rule::currency_code => "currency code (e.g., USD, EUR)".to_string(),
        Rule::buy_sell_args => "BUY/SELL arguments (ticker amount @ price)".to_string(),
        Rule::dividend_args => "DIVIDEND arguments".to_string(),
        Rule::capreturn_args => "CAPRETURN arguments".to_string(),
        Rule::split_args => "SPLIT arguments (ticker RATIO value)".to_string(),
        Rule::EOI => "end of input".to_string(),
        _ => format!("{rule:?}"),
    }
}

/// Parse a CGT file without FX conversion (all amounts must be GBP).
pub fn parse_file(input: &str) -> Result<Vec<Transaction>, CgtError> {
    parse_file_with_fx(input, None)
}

/// Parse a CGT file with optional FX conversion.
pub fn parse_file_with_fx(
    input: &str,
    fx_cache: Option<&FxCache>,
) -> Result<Vec<Transaction>, CgtError> {
    let pairs = CgtParser::parse(Rule::transaction_list, input).map_err(|err| {
        let ctx = from_pest_error(&err, input);
        CgtError::ParseErrorContext(ctx)
    })?;

    let mut transactions = Vec::new();
    let ctx = ParseContext { fx_cache };

    for pair in pairs {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::transaction {
                transactions.push(parse_transaction(inner, &ctx)?);
            }
        }
    }

    Ok(transactions)
}

struct ParseContext<'a> {
    fx_cache: Option<&'a FxCache>,
}

fn parse_transaction(
    pair: pest::iterators::Pair<Rule>,
    ctx: &ParseContext,
) -> Result<Transaction, CgtError> {
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
        Rule::cmd_buy => parse_buy_sell(command_inner, true, date, ctx)?,
        Rule::cmd_sell => parse_buy_sell(command_inner, false, date, ctx)?,
        Rule::cmd_dividend => parse_dividend(command_inner, date, ctx)?,
        Rule::cmd_capreturn => parse_capreturn(command_inner, date, ctx)?,
        Rule::cmd_split => parse_split(command_inner, true)?,
        Rule::cmd_unsplit => parse_split(command_inner, false)?,
        _ => return Err(CgtError::InvalidTransaction("Unknown command".to_string())),
    };

    Ok(Transaction {
        date,
        ticker,
        operation,
    })
}

fn parse_decimal(s: &str) -> Result<Decimal, CgtError> {
    Decimal::from_str(s).map_err(|_| CgtError::InvalidTransaction(format!("Invalid decimal: {s}")))
}

/// Parse a money rule (decimal with optional currency).
fn parse_money(
    pair: pest::iterators::Pair<Rule>,
    date: NaiveDate,
    ctx: &ParseContext,
) -> Result<CurrencyAmount, CgtError> {
    let mut inner = pair.into_inner();

    let amount = parse_decimal(
        inner
            .next()
            .ok_or(CgtError::UnexpectedParserState {
                expected: "decimal amount",
            })?
            .as_str(),
    )?;

    // Check for optional currency code
    let currency_code = inner.next().map(|p| p.as_str().to_uppercase());

    match currency_code {
        None => {
            // Default to GBP
            Ok(CurrencyAmount::gbp(amount))
        }
        Some(code) => {
            let currency = Currency::from_code(&code)
                .ok_or(CgtError::InvalidCurrencyCode { code: code.clone() })?;

            if currency == Currency::GBP {
                Ok(CurrencyAmount::gbp(amount))
            } else {
                // Need FX conversion
                let fx_cache = ctx.fx_cache.ok_or(CgtError::MissingFxRate {
                    currency: code.clone(),
                    year: date.year(),
                    month: date.month(),
                })?;

                let rate_entry = fx_cache.get(&code, date.year(), date.month()).ok_or(
                    CgtError::MissingFxRate {
                        currency: code.clone(),
                        year: date.year(),
                        month: date.month(),
                    },
                )?;

                let gbp = amount * rate_entry.rate_to_gbp;
                Ok(CurrencyAmount::foreign(amount, currency, gbp))
            }
        }
    }
}

fn parse_buy_sell(
    pair: pest::iterators::Pair<Rule>,
    is_buy: bool,
    date: NaiveDate,
    ctx: &ParseContext,
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
            .ok_or(CgtError::UnexpectedParserState {
                expected: "quantity",
            })?
            .as_str(),
    )?;

    let price_pair = inner
        .next()
        .ok_or(CgtError::UnexpectedParserState { expected: "price" })?;
    let price = parse_money(price_pair, date, ctx)?;

    // Optional expenses clause
    let expenses = if let Some(expenses_clause) = inner.next() {
        let money_pair =
            expenses_clause
                .into_inner()
                .next()
                .ok_or(CgtError::UnexpectedParserState {
                    expected: "expenses amount",
                })?;
        parse_money(money_pair, date, ctx)?
    } else {
        CurrencyAmount::gbp(Decimal::ZERO)
    };

    let operation = if is_buy {
        Operation::Buy {
            amount,
            price,
            expenses,
        }
    } else {
        Operation::Sell {
            amount,
            price,
            expenses,
        }
    };

    Ok((ticker, operation))
}

fn parse_dividend(
    pair: pest::iterators::Pair<Rule>,
    date: NaiveDate,
    ctx: &ParseContext,
) -> Result<(String, Operation), CgtError> {
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
            .ok_or(CgtError::UnexpectedParserState {
                expected: "quantity",
            })?
            .as_str(),
    )?;

    let total_value_pair = inner.next().ok_or(CgtError::UnexpectedParserState {
        expected: "total value",
    })?;
    let total_value = parse_money(total_value_pair, date, ctx)?;

    let tax_paid_pair = inner.next().ok_or(CgtError::UnexpectedParserState {
        expected: "tax paid",
    })?;
    let tax_paid = parse_money(tax_paid_pair, date, ctx)?;

    Ok((
        ticker,
        Operation::Dividend {
            amount,
            total_value,
            tax_paid,
        },
    ))
}

fn parse_capreturn(
    pair: pest::iterators::Pair<Rule>,
    date: NaiveDate,
    ctx: &ParseContext,
) -> Result<(String, Operation), CgtError> {
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
            .ok_or(CgtError::UnexpectedParserState {
                expected: "quantity",
            })?
            .as_str(),
    )?;

    let total_value_pair = inner.next().ok_or(CgtError::UnexpectedParserState {
        expected: "total value",
    })?;
    let total_value = parse_money(total_value_pair, date, ctx)?;

    let expenses_pair = inner.next().ok_or(CgtError::UnexpectedParserState {
        expected: "expenses",
    })?;
    let expenses = parse_money(expenses_pair, date, ctx)?;

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
    is_split: bool,
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

    let operation = if is_split {
        Operation::Split { ratio }
    } else {
        Operation::Unsplit { ratio }
    };

    Ok((ticker, operation))
}
