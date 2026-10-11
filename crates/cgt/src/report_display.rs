//! Prepared values and relevant notes shared by every report renderer.

use chrono::NaiveDate;
use num_bigint::BigInt;
use rust_decimal::Decimal;
use serde::Serialize;
use thiserror::Error;

use crate::format::{
    format_currency_amount, format_date, format_decimal_trimmed, format_gbp, format_gbp_unit_cost,
    format_price, format_unit_cost, round_money,
};
use crate::model::Operation;
use crate::report::{Disposal, MatchRule, TaxReport, TaxYearSummary};

/// A complete, immutable display document. Construct it with [`prepare`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ReportDisplay(DisplayData);

impl ReportDisplay {
    pub fn data(&self) -> &DisplayData {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayData {
    pub tax_years: Vec<DisplayedTaxYear>,
    pub holdings: Vec<DisplayedHolding>,
    pub trades: Vec<DisplayedTrade>,
    pub asset_events: Vec<DisplayedAssetEvent>,
    pub notes: Vec<ReportNote>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedTaxYear {
    pub period: String,
    pub disposal_count: String,
    pub total_gain: String,
    pub total_loss: String,
    pub net_gain: String,
    pub gross_proceeds: String,
    pub total_allowable_cost: String,
    pub exempt_amount: String,
    pub taxable_gain: String,
    pub dividend: Option<DisplayedDividend>,
    pub disposals: Vec<DisplayedDisposal>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedDividend {
    pub income: String,
    pub tax_paid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayDate {
    pub iso: String,
    pub uk: String,
}

impl From<NaiveDate> for DisplayDate {
    fn from(date: NaiveDate) -> Self {
        Self {
            iso: date.to_string(),
            uk: format_date(date),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnitCostText {
    pub money: String,
    pub compact: String,
}

impl UnitCostText {
    fn new(total: Decimal, quantity: Decimal) -> Self {
        let value = format_unit_cost(total, quantity);
        Self {
            money: format_gbp_unit_cost(total, quantity),
            compact: format!("£{}", value.trim_end_matches('0').trim_end_matches('.')),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResultKind {
    Gain,
    Loss,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedResult {
    pub kind: ResultKind,
    pub signed: String,
    pub magnitude: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedDisposal {
    pub date: DisplayDate,
    pub ticker: String,
    pub quantity: String,
    pub gross_proceeds: String,
    pub proceeds: String,
    pub sale_unit_price: String,
    pub fees: Option<String>,
    pub allowable_cost: String,
    pub result: DisplayedResult,
    pub rounding_adjustment: Option<String>,
    pub matches: Vec<DisplayedMatchLeg>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedMatchLeg {
    pub rule: MatchRule,
    pub rule_label: &'static str,
    pub quantity: String,
    pub allowable_cost: String,
    pub gain_or_loss: String,
    pub unit_cost: UnitCostText,
    pub acquisition_date: Option<DisplayDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedHolding {
    pub ticker: String,
    pub quantity: String,
    pub total_cost: String,
    pub unit_cost: UnitCostText,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedTrade {
    pub date: DisplayDate,
    pub action: &'static str,
    pub ticker: String,
    pub quantity: String,
    pub price: String,
    pub fees: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DisplayedAssetEvent {
    pub date: DisplayDate,
    pub action: &'static str,
    pub ticker: String,
    pub amount: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    DisposalCount,
    Gains,
    Proceeds,
    TaxableGain,
}

impl NoteKind {
    /// Stable identifier for layout markers and serialized notes.
    pub fn key(self) -> &'static str {
        match self {
            Self::DisposalCount => "disposal_count",
            Self::Gains => "gains",
            Self::Proceeds => "proceeds",
            Self::TaxableGain => "taxable_gain",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReportNote {
    pub kind: NoteKind,
    pub text: &'static str,
}

#[derive(Debug, Error)]
#[error("Report display failed: arithmetic exceeds the supported range or precision for {context}")]
pub struct DisplayError {
    context: String,
}

// Checked decimal addition can reduce its scale and discard fractional digits.
// Compare the result at a common scale before accepting it as an exact sum.
fn exact_add(left: Decimal, right: Decimal) -> Option<Decimal> {
    let sum = left.checked_add(right)?;
    let scale = left.scale().max(right.scale()).max(sum.scale());
    let scaled = |value: Decimal| {
        BigInt::from(value.mantissa()) * BigInt::from(10_u8).pow(scale - value.scale())
    };
    (scaled(sum) == scaled(left) + scaled(right)).then_some(sum)
}

fn exact_sub(left: Decimal, right: Decimal) -> Option<Decimal> {
    exact_add(left, -right)
}

fn arithmetic(
    value: Option<Decimal>,
    disposal: &Disposal,
    field: &str,
) -> Result<Decimal, DisplayError> {
    value.ok_or_else(|| DisplayError {
        context: format!("{field} of {} on {}", disposal.ticker, disposal.date),
    })
}

// Public reports can be constructed without the engine's arithmetic preflight.
// Establish the range and precision before using the shared rounding operations.
fn preflight(disposal: &Disposal) -> Result<(), DisplayError> {
    let mut cost = Decimal::ZERO;
    let mut gain = Decimal::ZERO;
    for leg in &disposal.legs {
        let next_cost = arithmetic(
            exact_add(cost, leg.allowable_cost),
            disposal,
            "matched cost",
        )?;
        let next_gain = arithmetic(
            exact_add(gain, leg.gain_or_loss),
            disposal,
            "matched result",
        )?;
        arithmetic(
            exact_sub(round_money(next_cost), round_money(cost)),
            disposal,
            "displayed match cost",
        )?;
        arithmetic(
            exact_sub(round_money(next_gain), round_money(gain)),
            disposal,
            "displayed match result",
        )?;
        cost = next_cost;
        gain = next_gain;
    }
    let independent = arithmetic(
        exact_sub(round_money(disposal.proceeds), round_money(cost)),
        disposal,
        "net proceeds less cost",
    )?;
    arithmetic(
        exact_sub(round_money(gain), independent),
        disposal,
        "rounding adjustment",
    )?;
    Ok(())
}

fn prepare_disposal(disposal: &Disposal) -> Result<DisplayedDisposal, DisplayError> {
    preflight(disposal)?;
    let unit_price = if disposal.quantity.is_zero() {
        Decimal::ZERO
    } else {
        arithmetic(
            disposal.gross_proceeds.checked_div(disposal.quantity),
            disposal,
            "sale unit price",
        )?
    };
    let precise_fees = arithmetic(
        exact_sub(disposal.gross_proceeds, disposal.proceeds),
        disposal,
        "sale fees",
    )?;
    let fees = if precise_fees > Decimal::ZERO {
        Some(format_gbp(arithmetic(
            exact_sub(
                round_money(disposal.gross_proceeds),
                round_money(disposal.proceeds),
            ),
            disposal,
            "displayed sale fees",
        )?))
    } else {
        None
    };
    let gain = disposal.total_gain();
    let adjustment = disposal.rounding_adjustment();
    let matches = disposal
        .display_legs()
        .into_iter()
        .map(|leg| DisplayedMatchLeg {
            rule_label: match leg.rule {
                MatchRule::SameDay => "Same Day",
                MatchRule::BedAndBreakfast => "Bed & Breakfast",
                MatchRule::Section104 => "Section 104",
            },
            rule: leg.rule,
            quantity: format_decimal_trimmed(leg.quantity),
            allowable_cost: format_gbp(leg.allowable_cost),
            gain_or_loss: format_gbp(leg.gain_or_loss),
            unit_cost: UnitCostText::new(leg.allowable_cost, leg.quantity),
            acquisition_date: leg.acquisition_date.map(DisplayDate::from),
        })
        .collect();
    Ok(DisplayedDisposal {
        date: disposal.date.into(),
        ticker: disposal.ticker.clone(),
        quantity: format_decimal_trimmed(disposal.quantity),
        gross_proceeds: format_gbp(disposal.gross_proceeds),
        proceeds: format_gbp(disposal.proceeds),
        sale_unit_price: format!("£{}", format_decimal_trimmed(unit_price)),
        fees,
        allowable_cost: format_gbp(disposal.total_allowable_cost()),
        result: DisplayedResult {
            kind: if gain >= Decimal::ZERO {
                ResultKind::Gain
            } else {
                ResultKind::Loss
            },
            signed: format_gbp(gain),
            magnitude: format_gbp(gain.abs()),
        },
        rounding_adjustment: (!adjustment.is_zero()).then(|| format_gbp(adjustment)),
        matches,
    })
}

fn prepare_year(year: &TaxYearSummary) -> Result<DisplayedTaxYear, DisplayError> {
    Ok(DisplayedTaxYear {
        period: year.period.to_string(),
        disposal_count: year.disposal_count.to_string(),
        total_gain: format_gbp(year.total_gain),
        total_loss: format_gbp(year.total_loss),
        net_gain: format_gbp(year.net_gain),
        gross_proceeds: format_gbp(year.gross_proceeds),
        total_allowable_cost: format_gbp(year.total_allowable_cost),
        exempt_amount: format_gbp(year.exempt_amount),
        taxable_gain: format_gbp(year.taxable_gain),
        dividend: (!year.dividend_income.is_zero() || !year.dividend_tax_paid.is_zero()).then(
            || DisplayedDividend {
                income: format_gbp(year.dividend_income),
                tax_paid: (!year.dividend_tax_paid.is_zero())
                    .then(|| format_gbp(year.dividend_tax_paid)),
            },
        ),
        disposals: year
            .disposals
            .iter()
            .map(prepare_disposal)
            .collect::<Result<_, _>>()?,
    })
}

fn notes(report: &TaxReport) -> Vec<ReportNote> {
    let mut notes = Vec::new();
    if report
        .tax_years
        .iter()
        .any(|year| !year.disposals.is_empty())
    {
        notes.extend([
            ReportNote { kind: NoteKind::DisposalCount, text: "Disposal count groups same-day disposals into a single transaction (CG51560). The count can differ from raw SELL transactions." },
            ReportNote { kind: NoteKind::Gains, text: "Gains/Losses are net per disposal after matching rules (CG51560)." },
            ReportNote { kind: NoteKind::Proceeds, text: "Proceeds are gross amounts before sale fees." },
        ]);
    }
    if !report.tax_years.is_empty() {
        notes.push(ReportNote {
            kind: NoteKind::TaxableGain,
            text: "Taxable gain is before losses brought forward and other reliefs.",
        });
    }
    notes
}

/// Resolve amounts, section selection, ordering, and notes for every output.
/// The source report retains its exact financial values.
///
/// # Errors
/// Returns [`DisplayError`] if display arithmetic exceeds the decimal range
/// or loses precision in a sum or difference.
pub fn prepare(report: &TaxReport) -> Result<ReportDisplay, DisplayError> {
    let tax_years = report
        .tax_years
        .iter()
        .map(prepare_year)
        .collect::<Result<_, _>>()?;
    let holdings = report
        .active_holdings()
        .into_iter()
        .map(|holding| DisplayedHolding {
            ticker: holding.ticker.clone(),
            quantity: format_decimal_trimmed(holding.quantity),
            total_cost: format_gbp(holding.total_cost),
            unit_cost: UnitCostText::new(holding.total_cost, holding.quantity),
        })
        .collect();
    let mut echo: Vec<_> = report.transactions.iter().flatten().collect();
    echo.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.ticker.cmp(&b.ticker)));
    let mut trades = Vec::new();
    let mut asset_events = Vec::new();
    for transaction in echo {
        let date = DisplayDate::from(transaction.date);
        let ticker = transaction.ticker.clone();
        match &transaction.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            }
            | Operation::Sell {
                amount,
                price,
                fees,
            } => {
                trades.push(DisplayedTrade {
                    date,
                    action: if matches!(transaction.operation, Operation::Buy { .. }) {
                        "BUY"
                    } else {
                        "SELL"
                    },
                    ticker,
                    quantity: format_decimal_trimmed(*amount),
                    price: format_price(price),
                    fees: format_price(fees),
                });
            }
            operation => {
                let (action, amount, value) = match operation {
                    Operation::Dividend { total_value, .. } => {
                        ("DIVIDEND", None, Some(format_currency_amount(total_value)))
                    }
                    Operation::Accumulation {
                        amount,
                        total_value,
                        ..
                    } => (
                        "ACCUMULATION",
                        Some(format_decimal_trimmed(*amount)),
                        Some(format_currency_amount(total_value)),
                    ),
                    Operation::CapReturn {
                        amount,
                        total_value,
                        ..
                    } => (
                        "CAPRETURN",
                        Some(format_decimal_trimmed(*amount)),
                        Some(format_currency_amount(total_value)),
                    ),
                    Operation::Split { ratio } => {
                        ("SPLIT", Some(format_decimal_trimmed(*ratio)), None)
                    }
                    Operation::Unsplit { ratio } => {
                        ("UNSPLIT", Some(format_decimal_trimmed(*ratio)), None)
                    }
                    Operation::Buy { .. } | Operation::Sell { .. } => continue,
                };
                asset_events.push(DisplayedAssetEvent {
                    date,
                    ticker,
                    action,
                    amount,
                    value,
                });
            }
        }
    }
    Ok(ReportDisplay(DisplayData {
        tax_years,
        holdings,
        trades,
        asset_events,
        notes: notes(report),
        warnings: report.warnings.clone(),
    }))
}
