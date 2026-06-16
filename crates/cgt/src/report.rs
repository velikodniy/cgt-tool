//! Public CGT report model. Every derived number is computed once, here or in
//! the engine; serializers apply the single money rounding policy.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::model::{TaxPeriod, Transaction};

/// HMRC share matching rule for a disposal leg.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}

/// One matched leg of a disposal: a quantity matched under a single rule with
/// its allowable cost and resulting gain or loss.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchLeg {
    pub rule: MatchRule,
    pub quantity: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub acquisition_date: Option<NaiveDate>,
}

/// One disposal: all legs for a single merged sell event.
#[derive(Debug, Clone, PartialEq)]
pub struct Disposal {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub gross_proceeds: Decimal,
    pub proceeds: Decimal,
    pub legs: Vec<MatchLeg>,
}

/// Final Section 104 pool state for a ticker after the whole replay.
#[derive(Debug, Clone, PartialEq)]
pub struct Holding {
    pub ticker: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

/// Per-tax-year summary of disposals and dividend income.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxYearSummary {
    pub period: TaxPeriod,
    pub disposals: Vec<Disposal>,
    pub disposal_count: usize,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
    pub gross_proceeds: Decimal,
    pub total_allowable_cost: Decimal,
    pub exempt_amount: Decimal,
    pub taxable_gain: Decimal,
    pub dividend_income: Decimal,
    pub dividend_tax_paid: Decimal,
}

/// The complete CGT report: per-year summaries, final holdings, and an
/// optional echo of the input transactions.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Holding>,
    pub transactions: Option<Vec<Transaction>>,
}
