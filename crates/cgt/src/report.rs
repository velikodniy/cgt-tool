//! Public CGT report model. Every derived number is computed once, here or in
//! the engine; serializers apply the single money rounding policy.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::ser::{Serialize, SerializeSeq, SerializeStruct, Serializer};

use crate::format::round_money;
use crate::model::{TaxPeriod, Transaction};

/// Serialize a money [`Decimal`] as a 2dp string under the single rounding
/// policy. Decimals render as JSON strings (matching the currency convention),
/// so totals stay exact rather than passing through binary float.
fn serialize_money<S>(value: Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&round_money(value).to_string())
}

/// Serialize a quantity [`Decimal`] as its exact string. Quantities are not
/// rounded; share counts are exact.
fn serialize_quantity<S>(value: Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// HMRC share matching rule for a disposal leg.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}

impl MatchRule {
    /// Wire name for this rule. The equivalence harness matches these exact
    /// strings; load-bearing for output equivalence.
    fn as_str(&self) -> &'static str {
        match self {
            MatchRule::SameDay => "SameDay",
            MatchRule::BedAndBreakfast => "BedAndBreakfast",
            MatchRule::Section104 => "Section104",
        }
    }
}

impl Serialize for MatchRule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
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

impl Serialize for MatchLeg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = if self.acquisition_date.is_some() {
            5
        } else {
            4
        };
        let mut leg = serializer.serialize_struct("MatchLeg", len)?;
        leg.serialize_field("rule", &self.rule)?;
        leg.serialize_field("quantity", &MoneyField(self.quantity, false))?;
        leg.serialize_field("allowable_cost", &MoneyField(self.allowable_cost, true))?;
        leg.serialize_field("gain_or_loss", &MoneyField(self.gain_or_loss, true))?;
        if let Some(date) = self.acquisition_date {
            leg.serialize_field("acquisition_date", &date)?;
        }
        leg.end()
    }
}

/// Serialize adapter for a single [`Decimal`] field. `round` selects the money
/// policy (2dp away-from-zero) when true, or the exact quantity form when
/// false.
struct MoneyField(Decimal, bool);

impl Serialize for MoneyField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.1 {
            serialize_money(self.0, serializer)
        } else {
            serialize_quantity(self.0, serializer)
        }
    }
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

impl Disposal {
    /// Net gain or loss across all legs of this disposal.
    pub fn total_gain(&self) -> Decimal {
        self.legs.iter().map(|leg| leg.gain_or_loss).sum()
    }

    /// Total allowable cost across all legs of this disposal.
    pub fn total_allowable_cost(&self) -> Decimal {
        self.legs.iter().map(|leg| leg.allowable_cost).sum()
    }
}

impl Serialize for Disposal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut disposal = serializer.serialize_struct("Disposal", 6)?;
        disposal.serialize_field("date", &self.date)?;
        disposal.serialize_field("ticker", &self.ticker)?;
        disposal.serialize_field("quantity", &MoneyField(self.quantity, false))?;
        disposal.serialize_field("gross_proceeds", &MoneyField(self.gross_proceeds, true))?;
        disposal.serialize_field("proceeds", &MoneyField(self.proceeds, true))?;
        // The legs are emitted under `matches` (the equivalence harness reads
        // `matches` or `legs`; this matches the existing wire vocabulary).
        disposal.serialize_field("matches", &Legs(&self.legs))?;
        disposal.end()
    }
}

/// Serialize adapter for a slice of [`MatchLeg`] as a JSON array.
struct Legs<'a>(&'a [MatchLeg]);

impl Serialize for Legs<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for leg in self.0 {
            seq.serialize_element(leg)?;
        }
        seq.end()
    }
}

/// Final Section 104 pool state for a ticker after the whole replay.
#[derive(Debug, Clone, PartialEq)]
pub struct Holding {
    pub ticker: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

impl Serialize for Holding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut holding = serializer.serialize_struct("Holding", 3)?;
        holding.serialize_field("ticker", &self.ticker)?;
        holding.serialize_field("quantity", &MoneyField(self.quantity, false))?;
        holding.serialize_field("total_cost", &MoneyField(self.total_cost, true))?;
        holding.end()
    }
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

impl Serialize for TaxYearSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut year = serializer.serialize_struct("TaxYearSummary", 12)?;
        year.serialize_field("period", &self.period)?;
        year.serialize_field("disposals", &self.disposals)?;
        year.serialize_field("disposal_count", &self.disposal_count)?;
        year.serialize_field("total_gain", &MoneyField(self.total_gain, true))?;
        year.serialize_field("total_loss", &MoneyField(self.total_loss, true))?;
        year.serialize_field("net_gain", &MoneyField(self.net_gain, true))?;
        // Computed-once renderer fields; outside the harness year view.
        year.serialize_field("gross_proceeds", &MoneyField(self.gross_proceeds, true))?;
        year.serialize_field(
            "total_allowable_cost",
            &MoneyField(self.total_allowable_cost, true),
        )?;
        year.serialize_field("exempt_amount", &MoneyField(self.exempt_amount, true))?;
        year.serialize_field("taxable_gain", &MoneyField(self.taxable_gain, true))?;
        year.serialize_field("dividend_income", &MoneyField(self.dividend_income, true))?;
        year.serialize_field(
            "dividend_tax_paid",
            &MoneyField(self.dividend_tax_paid, true),
        )?;
        year.end()
    }
}

/// The complete CGT report: per-year summaries, final holdings, an optional
/// echo of the input transactions, and any non-fatal warnings.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxReport {
    pub tax_years: Vec<TaxYearSummary>,
    pub holdings: Vec<Holding>,
    pub transactions: Option<Vec<Transaction>>,
    /// Non-fatal diagnostics (e.g. a tax year with no configured exemption).
    pub warnings: Vec<String>,
}

impl Serialize for TaxReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = if self.transactions.is_some() { 3 } else { 2 };
        if !self.warnings.is_empty() {
            len += 1;
        }
        let mut report = serializer.serialize_struct("TaxReport", len)?;
        report.serialize_field("tax_years", &self.tax_years)?;
        report.serialize_field("holdings", &self.holdings)?;
        if let Some(transactions) = &self.transactions {
            report.serialize_field("transactions", transactions)?;
        }
        if !self.warnings.is_empty() {
            report.serialize_field("warnings", &self.warnings)?;
        }
        report.end()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use serde_json::Value;

    use super::{Disposal, Holding, MatchLeg, MatchRule, TaxPeriod, TaxReport, TaxYearSummary};

    fn sample_report() -> TaxReport {
        TaxReport {
            tax_years: vec![TaxYearSummary {
                period: TaxPeriod::new(2023).expect("valid tax year"),
                disposals: vec![Disposal {
                    date: NaiveDate::from_ymd_opt(2023, 9, 1).expect("valid date"),
                    ticker: "FOOBAR".to_string(),
                    quantity: dec!(10),
                    gross_proceeds: dec!(100.005),
                    proceeds: dec!(95.50),
                    legs: vec![MatchLeg {
                        rule: MatchRule::SameDay,
                        quantity: dec!(10),
                        allowable_cost: dec!(54.065),
                        gain_or_loss: dec!(-19.855),
                        acquisition_date: Some(
                            NaiveDate::from_ymd_opt(2023, 9, 1).expect("valid date"),
                        ),
                    }],
                }],
                disposal_count: 1,
                total_gain: dec!(0),
                total_loss: dec!(19.855),
                net_gain: dec!(-19.855),
                gross_proceeds: dec!(100.005),
                total_allowable_cost: dec!(54.065),
                exempt_amount: dec!(11700),
                taxable_gain: dec!(0),
                dividend_income: dec!(0),
                dividend_tax_paid: dec!(0),
            }],
            holdings: vec![Holding {
                ticker: "FOOBAR".to_string(),
                quantity: dec!(6),
                total_cost: dec!(1774.005),
            }],
            transactions: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn money_serializes_2dp_with_midpoint_away_from_zero() {
        let json = serde_json::to_value(sample_report()).expect("serializes");
        let leg = &json["tax_years"][0]["disposals"][0]["matches"][0];

        // .xx5 midpoints round away from zero, not to even.
        assert_eq!(leg["allowable_cost"], Value::from("54.07"));
        assert_eq!(leg["gain_or_loss"], Value::from("-19.86"));
        let disposal = &json["tax_years"][0]["disposals"][0];
        assert_eq!(disposal["gross_proceeds"], Value::from("100.01"));
    }

    #[test]
    fn field_names_and_rule_strings_match_the_wire_schema() {
        let json = serde_json::to_value(sample_report()).expect("serializes");
        let year = &json["tax_years"][0];

        // Year emits both harness-read leaves and renderer-only extras.
        for key in [
            "period",
            "disposals",
            "disposal_count",
            "total_gain",
            "total_loss",
            "net_gain",
            "gross_proceeds",
            "total_allowable_cost",
            "exempt_amount",
            "taxable_gain",
            "dividend_income",
            "dividend_tax_paid",
        ] {
            assert!(year.get(key).is_some(), "year missing {key}");
        }
        assert_eq!(year["period"], Value::from("2023/24"));
        assert_eq!(year["disposal_count"], Value::from(1));

        let disposal = &year["disposals"][0];
        for key in [
            "date",
            "ticker",
            "quantity",
            "gross_proceeds",
            "proceeds",
            "matches",
        ] {
            assert!(disposal.get(key).is_some(), "disposal missing {key}");
        }
        // Quantities are exact, never rounded.
        assert_eq!(disposal["quantity"], Value::from("10"));

        let leg = &disposal["matches"][0];
        assert_eq!(leg["rule"], Value::from("SameDay"));
        assert_eq!(leg["acquisition_date"], Value::from("2023-09-01"));

        let holding = &json["holdings"][0];
        for key in ["ticker", "quantity", "total_cost"] {
            assert!(holding.get(key).is_some(), "holding missing {key}");
        }
        assert_eq!(holding["total_cost"], Value::from("1774.01"));

        // The input echo is skipped when absent.
        assert!(json.get("transactions").is_none());
    }

    #[test]
    fn acquisition_date_is_skipped_when_absent() {
        let mut report = sample_report();
        report.tax_years[0].disposals[0].legs[0].acquisition_date = None;
        let json = serde_json::to_value(report).expect("serializes");
        let leg = &json["tax_years"][0]["disposals"][0]["matches"][0];
        assert!(leg.get("acquisition_date").is_none());
    }

    #[test]
    fn match_rule_strings_are_exact() {
        assert_eq!(
            serde_json::to_value(MatchRule::BedAndBreakfast).expect("serializes"),
            Value::from("BedAndBreakfast")
        );
        assert_eq!(
            serde_json::to_value(MatchRule::Section104).expect("serializes"),
            Value::from("Section104")
        );
    }
}
