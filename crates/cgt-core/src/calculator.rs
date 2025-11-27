use crate::models::*;
use crate::error::CgtError;
use rust_decimal::Decimal;
// use rust_decimal::prelude::Zero; // Not used currently
// use std::cmp::Ordering; // Not used currently

pub fn calculate(
    mut transactions: Vec<Transaction>,
    tax_year_start: i32,
) -> Result<TaxReport, CgtError> {
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    // Merge same-day same-ticker transactions
    let mut merged = Vec::new();
    if !transactions.is_empty() {
        let mut current = transactions[0].clone();

        for next in transactions.into_iter().skip(1) {
            if next.date == current.date && next.ticker == current.ticker {
                match (&mut current.operation, next.operation) {
                    (
                        Operation::Buy {
                            amount: a1,
                            price: p1,
                            expenses: e1,
                        },
                        Operation::Buy {
                            amount: a2,
                            price: p2,
                            expenses: e2,
                        },
                    ) => {
                        let total_cost = (*a1 * *p1) + (a2 * p2);
                        *a1 += a2;
                        *p1 = total_cost / *a1;
                        *e1 += e2;
                    }
                    (
                        Operation::Sell {
                            amount: a1,
                            price: p1,
                            expenses: e1,
                        },
                        Operation::Sell {
                            amount: a2,
                            price: p2,
                            expenses: e2,
                        },
                    ) => {
                        let total_proceeds = (*a1 * *p1) + (a2 * p2);
                        *a1 += a2;
                        *p1 = total_proceeds / *a1;
                        *e1 += e2;
                    }
                    (_, next_op) => {
                        merged.push(current);
                        current = Transaction {
                            date: next.date,
                            ticker: next.ticker,
                            operation: next_op,
                        };
                    }
                }
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);
    }
    let transactions = merged;

    let mut matches = Vec::new();
    let mut pool = Section104Holding {
        ticker: "GLOBAL".to_string(),
        quantity: Decimal::ZERO,
        total_cost: Decimal::ZERO,
    };

    let mut consumed = vec![Decimal::ZERO; transactions.len()];

    // Pass 1: Same Day
    for i in 0..transactions.len() {
        if let Operation::Sell {
            amount: sell_qty, ..
        } = &transactions[i].operation
        {
            let mut remaining_sell = *sell_qty - consumed[i];
            if remaining_sell <= Decimal::ZERO {
                continue;
            }

            for j in 0..transactions.len() {
                if i == j {
                    continue;
                }
                if transactions[j].date != transactions[i].date {
                    continue;
                }

                if let Operation::Buy {
                    amount: buy_qty,
                    price: buy_price,
                    expenses: buy_exp,
                } = &transactions[j].operation
                {
                    let remaining_buy = *buy_qty - consumed[j];
                    if remaining_buy <= Decimal::ZERO {
                        continue;
                    }

                    let match_qty = remaining_sell.min(remaining_buy);
                    let _unit_cost = *buy_price + (*buy_exp / *buy_qty);
                    let cost_portion =
                        (match_qty * *buy_price) + (*buy_exp * (match_qty / *buy_qty));

                    consumed[i] += match_qty;
                    consumed[j] += match_qty;
                    remaining_sell -= match_qty;

                    let proceeds = get_proceeds(&transactions[i], match_qty);
                    matches.push(Match {
                        date: transactions[i].date,
                        ticker: transactions[i].ticker.clone(),
                        quantity: match_qty,
                        proceeds,
                        allowable_cost: cost_portion,
                        gain_or_loss: proceeds - cost_portion,
                        rule: MatchRule::SameDay,
                    });

                    if remaining_sell <= Decimal::ZERO {
                        break;
                    }
                }
            }
        }
    }

    // Pass 2: Bed & Breakfast
    for i in 0..transactions.len() {
        if let Operation::Sell {
            amount: sell_qty, ..
        } = &transactions[i].operation
        {
            let mut remaining_sell = *sell_qty - consumed[i];
            if remaining_sell <= Decimal::ZERO {
                continue;
            }

            let mut ratio = Decimal::ONE;

            for j in (i + 1)..transactions.len() {
                let days_diff = (transactions[j].date - transactions[i].date).num_days();
                if days_diff > 30 {
                    break;
                }

                match &transactions[j].operation {
                    Operation::Split { ratio: r } => ratio *= r,
                    Operation::Unsplit { ratio: r } => ratio /= r,
                    Operation::Buy {
                        amount: buy_qty,
                        price: buy_price,
                        expenses: buy_exp,
                    } => {
                        // Buy is strictly future (B&B). If same day, handled by Pass 1.
                        if days_diff <= 0 {
                            continue;
                        }

                        // buy_qty is in "j" units. Convert to "i" units: buy_qty / ratio.
                        let available_buy_j = *buy_qty - consumed[j];
                        if available_buy_j <= Decimal::ZERO {
                            continue;
                        }

                        // Available buy in "i" units
                        let available_buy_i = available_buy_j / ratio;

                        let match_qty_i = remaining_sell.min(available_buy_i);

                        // Convert match back to "j" units for consumption/cost
                        let match_qty_j = match_qty_i * ratio;

                        let cost_portion =
                            (match_qty_j * *buy_price) + (*buy_exp * (match_qty_j / *buy_qty));

                        consumed[i] += match_qty_i;
                        consumed[j] += match_qty_j;
                        remaining_sell -= match_qty_i;

                        let proceeds = get_proceeds(&transactions[i], match_qty_i);
                        matches.push(Match {
                            date: transactions[i].date,
                            ticker: transactions[i].ticker.clone(),
                            quantity: match_qty_i,
                            proceeds,
                            allowable_cost: cost_portion,
                            gain_or_loss: proceeds - cost_portion,
                            rule: MatchRule::BedAndBreakfast,
                        });

                        if remaining_sell <= Decimal::ZERO {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Pass 3: Section 104
    for i in 0..transactions.len() {
        let t = &transactions[i];
        match &t.operation {
            Operation::Buy {
                amount,
                price,
                expenses,
            } => {
                let remaining = *amount - consumed[i];
                if remaining > Decimal::ZERO {
                    pool.quantity += remaining;
                    let cost_add = (remaining * *price) + (*expenses * (remaining / *amount));
                    pool.total_cost += cost_add;
                }
            }
            Operation::Sell { amount, .. } => {
                let remaining = *amount - consumed[i];
                if remaining > Decimal::ZERO {
                    if pool.quantity <= Decimal::ZERO {
                        return Err(CgtError::InvalidTransaction(format!(
                            "Sell of {} {} on {} exceeds holding (Pool: {})",
                            remaining, t.ticker, t.date, pool.quantity
                        )));
                    }

                    let match_qty = remaining;
                    let cost_portion = pool.total_cost * (match_qty / pool.quantity);

                    pool.quantity -= match_qty;
                    pool.total_cost -= cost_portion;

                    let proceeds = get_proceeds(t, match_qty);
                    matches.push(Match {
                        date: t.date,
                        ticker: t.ticker.clone(),
                        quantity: match_qty,
                        proceeds,
                        allowable_cost: cost_portion,
                        gain_or_loss: proceeds - cost_portion,
                        rule: MatchRule::Section104,
                    });
                }
            }
            Operation::Split { ratio } => {
                pool.quantity *= *ratio;
            }
            Operation::Unsplit { ratio } => {
                pool.quantity /= *ratio;
            }
            Operation::CapReturn { amount, expenses } => {
                let net_return = *amount - *expenses;
                pool.total_cost -= net_return;
            }
            Operation::Dividend { .. } => {}
        }

        if pool.ticker == "GLOBAL" {
            pool.ticker = t.ticker.clone();
        }
    }

    let start_date = chrono::NaiveDate::from_ymd_opt(tax_year_start, 4, 6).unwrap();
    let end_date = chrono::NaiveDate::from_ymd_opt(tax_year_start + 1, 4, 5).unwrap();

    let year_matches: Vec<Match> = matches
        .into_iter()
        .filter(|m| m.date >= start_date && m.date <= end_date)
        .collect();

    let total_gain: Decimal = year_matches
        .iter()
        .map(|m| {
            if m.gain_or_loss > Decimal::ZERO {
                m.gain_or_loss
            } else {
                Decimal::ZERO
            }
        })
        .sum();
    let total_loss: Decimal = year_matches
        .iter()
        .map(|m| {
            if m.gain_or_loss < Decimal::ZERO {
                m.gain_or_loss.abs()
            } else {
                Decimal::ZERO
            }
        })
        .sum();

    Ok(TaxReport {
        tax_year: tax_year_start,
        matches: year_matches,
        total_gain,
        total_loss,
        net_gain: total_gain - total_loss,
        holdings: vec![pool],
    })
}

fn get_proceeds(t: &Transaction, qty: Decimal) -> Decimal {
    if let Operation::Sell {
        amount,
        price,
        expenses,
    } = &t.operation
    {
        let gross = qty * *price;
        let exp_portion = *expenses * (qty / *amount);
        gross - exp_portion
    } else {
        Decimal::ZERO
    }
}
