use chrono::NaiveDate;
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Transaction {
    pub date: NaiveDate,
    pub ticker: String,
    #[serde(flatten)]
    pub operation: Operation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operation {
    Buy {
        amount: Decimal,
        price: Decimal,
        expenses: Decimal,
    },
    Sell {
        amount: Decimal,
        price: Decimal,
        expenses: Decimal,
    },
    Dividend {
        amount: Decimal,
        total_value: Decimal,
        tax_paid: Decimal,
    },
    #[serde(rename = "CAPRETURN")]
    CapReturn {
        amount: Decimal,
        total_value: Decimal,
        expenses: Decimal,
    },
    Split {
        ratio: Decimal,
    },
    Unsplit {
        ratio: Decimal,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Section104Holding {
    pub ticker: String,
    pub quantity: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Match {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub rule: MatchRule,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum MatchRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct TaxReport {
    pub tax_year: i32,
    pub matches: Vec<Match>,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
    pub holdings: Vec<Section104Holding>,
}
