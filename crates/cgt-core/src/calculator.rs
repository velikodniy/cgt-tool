use crate::error::CgtError;
use crate::models::*;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Tracks an individual acquisition with cost adjustments from CAPRETURN/DIVIDEND events
#[derive(Debug, Clone)]
struct AcquisitionTracker {
    amount: Decimal,
    price: Decimal,
    expenses: Decimal,
    cost_offset: Decimal, // Positive for DIVIDEND (increases cost), negative for CAPRETURN (reduces cost)
}

impl AcquisitionTracker {
    fn adjusted_cost(&self) -> Decimal {
        let base_cost = (self.amount * self.price) + self.expenses;
        base_cost + self.cost_offset
    }

    fn adjusted_unit_cost(&self) -> Decimal {
        self.adjusted_cost() / self.amount
    }
}

/// Internal match representation during calculation (before grouping into Disposals)
#[derive(Debug, Clone)]
struct InternalMatch {
    disposal_date: NaiveDate,
    disposal_ticker: String,
    quantity: Decimal,
    proceeds: Decimal,
    allowable_cost: Decimal,
    gain_or_loss: Decimal,
    rule: MatchRule,
    acquisition_date: Option<NaiveDate>,
}

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
                            amount: current_amount,
                            price: current_price,
                            expenses: current_expenses,
                        },
                        Operation::Buy {
                            amount: next_amount,
                            price: next_price,
                            expenses: next_expenses,
                        },
                    ) => {
                        let total_cost =
                            (*current_amount * *current_price) + (next_amount * next_price);
                        *current_amount += next_amount;
                        *current_price = total_cost / *current_amount;
                        *current_expenses += next_expenses;
                    }
                    (
                        Operation::Sell {
                            amount: current_amount,
                            price: current_price,
                            expenses: current_expenses,
                        },
                        Operation::Sell {
                            amount: next_amount,
                            price: next_price,
                            expenses: next_expenses,
                        },
                    ) => {
                        let total_proceeds =
                            (*current_amount * *current_price) + (next_amount * next_price);
                        *current_amount += next_amount;
                        *current_price = total_proceeds / *current_amount;
                        *current_expenses += next_expenses;
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

    // Create acquisition trackers for all BUY transactions
    let mut acquisition_trackers: Vec<Option<AcquisitionTracker>> = transactions
        .iter()
        .map(|tx| match &tx.operation {
            Operation::Buy {
                amount,
                price,
                expenses,
            } => Some(AcquisitionTracker {
                amount: *amount,
                price: *price,
                expenses: *expenses,
                cost_offset: Decimal::ZERO,
            }),
            _ => None,
        })
        .collect();

    // Preprocessing Pass 1: Apply CAPRETURN events to reduce acquisition costs
    for (event_idx, tx) in transactions.iter().enumerate() {
        if let Operation::CapReturn {
            amount: event_amount,
            total_value,
            expenses: event_expenses,
        } = &tx.operation
        {
            // Track how much of each acquisition is left after sells before this event
            let mut acquisition_amounts_left: Vec<Decimal> = acquisition_trackers
                .iter()
                .map(|opt| opt.as_ref().map(|acq| acq.amount).unwrap_or(Decimal::ZERO))
                .collect();

            // Process all transactions before this event chronologically to track what's left
            for (before_idx, before_tx) in transactions.iter().enumerate().take(event_idx) {
                if let Operation::Sell { amount, .. } = &before_tx.operation
                    && before_tx.date < tx.date
                {
                    let mut remaining_to_match = *amount;
                    // Match against acquisitions in FIFO order
                    for amount_left in acquisition_amounts_left.iter_mut().take(before_idx) {
                        if remaining_to_match <= Decimal::ZERO {
                            break;
                        }
                        if *amount_left > Decimal::ZERO {
                            let matched = remaining_to_match.min(*amount_left);
                            *amount_left -= matched;
                            remaining_to_match -= matched;
                        }
                    }
                }
            }

            // Apportion the capital return value to acquisitions based on amounts left
            let net_value = *total_value - *event_expenses;
            for (acq_idx, acq_opt) in acquisition_trackers.iter_mut().enumerate() {
                if acq_idx >= event_idx {
                    break;
                }
                if let Some(acq) = acq_opt {
                    let amount_left = acquisition_amounts_left[acq_idx];
                    if amount_left > Decimal::ZERO && transactions[acq_idx].date < tx.date {
                        let apportioned_value = net_value * (amount_left / event_amount);
                        acq.cost_offset -= apportioned_value; // Reduce cost
                    }
                }
            }
        }
    }

    // Preprocessing Pass 2: Apply DIVIDEND events to increase acquisition costs
    for (event_idx, tx) in transactions.iter().enumerate() {
        if let Operation::Dividend {
            amount: event_amount,
            total_value,
            tax_paid: _,
        } = &tx.operation
        {
            // Track how much of each acquisition is left after sells before this event
            let mut acquisition_amounts_left: Vec<Decimal> = acquisition_trackers
                .iter()
                .map(|opt| opt.as_ref().map(|acq| acq.amount).unwrap_or(Decimal::ZERO))
                .collect();

            // Process all transactions before this event chronologically to track what's left
            for (before_idx, before_tx) in transactions.iter().enumerate().take(event_idx) {
                if let Operation::Sell { amount, .. } = &before_tx.operation
                    && before_tx.date < tx.date
                {
                    let mut remaining_to_match = *amount;
                    // Match against acquisitions in FIFO order
                    for amount_left in acquisition_amounts_left.iter_mut().take(before_idx) {
                        if remaining_to_match <= Decimal::ZERO {
                            break;
                        }
                        if *amount_left > Decimal::ZERO {
                            let matched = remaining_to_match.min(*amount_left);
                            *amount_left -= matched;
                            remaining_to_match -= matched;
                        }
                    }
                }
            }

            // Apportion the dividend value to acquisitions based on amounts left
            // Note: For dividends, the value is after tax, so we don't adjust for tax_paid
            let net_value = *total_value;
            for (acq_idx, acq_opt) in acquisition_trackers.iter_mut().enumerate() {
                if acq_idx >= event_idx {
                    break;
                }
                if let Some(acq) = acq_opt {
                    let amount_left = acquisition_amounts_left[acq_idx];
                    if amount_left > Decimal::ZERO && transactions[acq_idx].date < tx.date {
                        let apportioned_value = net_value * (amount_left / event_amount);
                        acq.cost_offset += apportioned_value; // Increase cost
                    }
                }
            }
        }
    }

    let mut internal_matches = Vec::new();
    let mut pool = Section104Holding {
        ticker: "GLOBAL".to_string(),
        quantity: Decimal::ZERO,
        total_cost: Decimal::ZERO,
    };

    let mut consumed = vec![Decimal::ZERO; transactions.len()];

    // Pass 1: Same Day
    for sell_transaction_idx in 0..transactions.len() {
        if let Operation::Sell {
            amount: sell_amount,
            ..
        } = &transactions[sell_transaction_idx].operation
        {
            let mut remaining_sell_amount = *sell_amount - consumed[sell_transaction_idx];
            if remaining_sell_amount <= Decimal::ZERO {
                continue;
            }

            for buy_transaction_idx in 0..transactions.len() {
                if sell_transaction_idx == buy_transaction_idx {
                    continue;
                }
                if transactions[buy_transaction_idx].date != transactions[sell_transaction_idx].date
                {
                    continue;
                }

                if let Operation::Buy {
                    amount: buy_amount, ..
                } = &transactions[buy_transaction_idx].operation
                {
                    let remaining_buy_amount = *buy_amount - consumed[buy_transaction_idx];
                    if remaining_buy_amount <= Decimal::ZERO {
                        continue;
                    }

                    let matched_quantity = remaining_sell_amount.min(remaining_buy_amount);

                    // Use adjusted cost from tracker
                    let cost_portion =
                        if let Some(tracker) = &acquisition_trackers[buy_transaction_idx] {
                            matched_quantity * tracker.adjusted_unit_cost()
                        } else {
                            return Err(CgtError::InvalidTransaction(
                                "Missing acquisition tracker for BUY transaction".to_string(),
                            ));
                        };

                    consumed[sell_transaction_idx] += matched_quantity;
                    consumed[buy_transaction_idx] += matched_quantity;
                    remaining_sell_amount -= matched_quantity;

                    let proceeds =
                        get_proceeds(&transactions[sell_transaction_idx], matched_quantity);
                    internal_matches.push(InternalMatch {
                        disposal_date: transactions[sell_transaction_idx].date,
                        disposal_ticker: transactions[sell_transaction_idx].ticker.clone(),
                        quantity: matched_quantity,
                        proceeds,
                        allowable_cost: cost_portion,
                        gain_or_loss: proceeds - cost_portion,
                        rule: MatchRule::SameDay,
                        acquisition_date: None, // Same day - no separate acquisition date needed
                    });

                    if remaining_sell_amount <= Decimal::ZERO {
                        break;
                    }
                }
            }
        }
    }

    // Pass 2: Bed & Breakfast
    for sell_transaction_idx in 0..transactions.len() {
        if let Operation::Sell {
            amount: sell_amount,
            ..
        } = &transactions[sell_transaction_idx].operation
        {
            let mut remaining_sell_amount = *sell_amount - consumed[sell_transaction_idx];
            if remaining_sell_amount <= Decimal::ZERO {
                continue;
            }

            let mut cumulative_ratio_effect = Decimal::ONE;

            for buy_transaction_idx in (sell_transaction_idx + 1)..transactions.len() {
                let days_diff = (transactions[buy_transaction_idx].date
                    - transactions[sell_transaction_idx].date)
                    .num_days();
                if days_diff > 30 {
                    break;
                }

                match &transactions[buy_transaction_idx].operation {
                    Operation::Split { ratio: split_ratio } => {
                        cumulative_ratio_effect *= split_ratio
                    }
                    Operation::Unsplit {
                        ratio: unsplit_ratio,
                    } => cumulative_ratio_effect /= unsplit_ratio,
                    Operation::Buy {
                        amount: buy_amount, ..
                    } => {
                        if days_diff <= 0 {
                            continue;
                        }

                        let available_buy_amount_at_buy_time =
                            *buy_amount - consumed[buy_transaction_idx];
                        if available_buy_amount_at_buy_time <= Decimal::ZERO {
                            continue;
                        }

                        let available_buy_amount_at_sell_time =
                            available_buy_amount_at_buy_time / cumulative_ratio_effect;

                        let matched_quantity_at_sell_time =
                            remaining_sell_amount.min(available_buy_amount_at_sell_time);

                        let matched_quantity_at_buy_time =
                            matched_quantity_at_sell_time * cumulative_ratio_effect;

                        // Use adjusted cost from tracker
                        let cost_portion =
                            if let Some(tracker) = &acquisition_trackers[buy_transaction_idx] {
                                matched_quantity_at_buy_time * tracker.adjusted_unit_cost()
                            } else {
                                return Err(CgtError::InvalidTransaction(
                                    "Missing acquisition tracker for BUY transaction".to_string(),
                                ));
                            };

                        consumed[sell_transaction_idx] += matched_quantity_at_sell_time;
                        consumed[buy_transaction_idx] += matched_quantity_at_buy_time;
                        remaining_sell_amount -= matched_quantity_at_sell_time;

                        let proceeds = get_proceeds(
                            &transactions[sell_transaction_idx],
                            matched_quantity_at_sell_time,
                        );
                        internal_matches.push(InternalMatch {
                            disposal_date: transactions[sell_transaction_idx].date,
                            disposal_ticker: transactions[sell_transaction_idx].ticker.clone(),
                            quantity: matched_quantity_at_sell_time,
                            proceeds,
                            allowable_cost: cost_portion,
                            gain_or_loss: proceeds - cost_portion,
                            rule: MatchRule::BedAndBreakfast,
                            acquisition_date: Some(transactions[buy_transaction_idx].date),
                        });

                        if remaining_sell_amount <= Decimal::ZERO {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Pass 3: Section 104
    for transaction_idx in 0..transactions.len() {
        let current_transaction = &transactions[transaction_idx];
        match &current_transaction.operation {
            Operation::Buy { amount, .. } => {
                let remaining_amount = *amount - consumed[transaction_idx];
                if remaining_amount > Decimal::ZERO {
                    pool.quantity += remaining_amount;
                    // Use adjusted cost from tracker
                    let cost_add = if let Some(tracker) = &acquisition_trackers[transaction_idx] {
                        remaining_amount * tracker.adjusted_unit_cost()
                    } else {
                        return Err(CgtError::InvalidTransaction(
                            "Missing acquisition tracker for BUY transaction".to_string(),
                        ));
                    };
                    pool.total_cost += cost_add;
                }
            }
            Operation::Sell { amount, .. } => {
                let remaining_amount = *amount - consumed[transaction_idx];
                if remaining_amount > Decimal::ZERO {
                    if pool.quantity <= Decimal::ZERO {
                        return Err(CgtError::InvalidTransaction(format!(
                            "Sell of {} {} on {} exceeds holding (Pool: {})",
                            remaining_amount,
                            current_transaction.ticker,
                            current_transaction.date,
                            pool.quantity
                        )));
                    }

                    let matched_quantity = remaining_amount;
                    let cost_portion = pool.total_cost * (matched_quantity / pool.quantity);

                    pool.quantity -= matched_quantity;
                    pool.total_cost -= cost_portion;

                    let proceeds = get_proceeds(current_transaction, matched_quantity);
                    internal_matches.push(InternalMatch {
                        disposal_date: current_transaction.date,
                        disposal_ticker: current_transaction.ticker.clone(),
                        quantity: matched_quantity,
                        proceeds,
                        allowable_cost: cost_portion,
                        gain_or_loss: proceeds - cost_portion,
                        rule: MatchRule::Section104,
                        acquisition_date: None, // Section 104 pool has no single acquisition date
                    });
                }
            }
            Operation::Split { ratio } => {
                pool.quantity *= *ratio;
            }
            Operation::Unsplit { ratio } => {
                pool.quantity /= *ratio;
            }
            // CAPRETURN and DIVIDEND are handled in preprocessing, nothing to do here
            Operation::CapReturn { .. } => {}
            Operation::Dividend { .. } => {}
        }

        if pool.ticker == "GLOBAL" {
            pool.ticker = current_transaction.ticker.clone();
        }
    }

    // Filter matches for the requested tax year
    let start_date =
        chrono::NaiveDate::from_ymd_opt(tax_year_start, 4, 6).ok_or(CgtError::InvalidDateYear {
            year: tax_year_start,
        })?;
    let end_date = chrono::NaiveDate::from_ymd_opt(tax_year_start + 1, 4, 5).ok_or(
        CgtError::InvalidDateYear {
            year: tax_year_start + 1,
        },
    )?;

    let year_matches: Vec<InternalMatch> = internal_matches
        .into_iter()
        .filter(|m| m.disposal_date >= start_date && m.disposal_date <= end_date)
        .collect();

    // Group matches into disposals by (date, ticker)
    let disposals = group_matches_into_disposals(year_matches);

    // Calculate totals
    let total_gain: Decimal = disposals
        .iter()
        .flat_map(|d| &d.matches)
        .map(|m| {
            if m.gain_or_loss > Decimal::ZERO {
                m.gain_or_loss
            } else {
                Decimal::ZERO
            }
        })
        .sum();
    let total_loss: Decimal = disposals
        .iter()
        .flat_map(|d| &d.matches)
        .map(|m| {
            if m.gain_or_loss < Decimal::ZERO {
                m.gain_or_loss.abs()
            } else {
                Decimal::ZERO
            }
        })
        .sum();

    // Create tax year summary
    let tax_period = TaxPeriod::from_date(start_date);
    let tax_year_summary = TaxYearSummary {
        period: tax_period,
        disposals,
        total_gain,
        total_loss,
        net_gain: total_gain - total_loss,
    };

    Ok(TaxReport {
        tax_years: vec![tax_year_summary],
        holdings: vec![pool],
    })
}

/// Group internal matches into Disposal objects by (date, ticker)
fn group_matches_into_disposals(internal_matches: Vec<InternalMatch>) -> Vec<Disposal> {
    // Group by (date, ticker)
    let mut disposal_map: HashMap<(NaiveDate, String), Vec<InternalMatch>> = HashMap::new();

    for m in internal_matches {
        let key = (m.disposal_date, m.disposal_ticker.clone());
        disposal_map.entry(key).or_default().push(m);
    }

    // Convert to Disposal structs
    let mut disposals: Vec<Disposal> = disposal_map
        .into_iter()
        .map(|((date, ticker), matches)| {
            let total_proceeds: Decimal = matches.iter().map(|m| m.proceeds).sum();
            let total_quantity: Decimal = matches.iter().map(|m| m.quantity).sum();

            let converted_matches: Vec<Match> = matches
                .into_iter()
                .map(|m| Match {
                    rule: m.rule,
                    quantity: m.quantity,
                    allowable_cost: m.allowable_cost,
                    gain_or_loss: m.gain_or_loss,
                    acquisition_date: m.acquisition_date,
                })
                .collect();

            Disposal {
                date,
                ticker,
                quantity: total_quantity,
                proceeds: total_proceeds,
                matches: converted_matches,
            }
        })
        .collect();

    // Sort disposals by date for consistent output
    disposals.sort_by(|a, b| a.date.cmp(&b.date));

    disposals
}

fn get_proceeds(current_transaction: &Transaction, qty: Decimal) -> Decimal {
    if let Operation::Sell {
        amount,
        price,
        expenses,
    } = &current_transaction.operation
    {
        let gross = qty * *price;
        let exp_portion = *expenses * (qty / *amount);
        gross - exp_portion
    } else {
        Decimal::ZERO
    }
}
