# Data Model: Capital Gains Tax (CGT) CLI Tool

**Feature**: `001-cgt-cli`
**Date**: 2025-11-27

## Entities

### 1. Transaction

Represents a single parsed event. We use an algebraic data type (Enum) to represent the different operations, as they require different data fields.

```rust
// crates/cgt-core/src/models.rs

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub date: NaiveDate,
    pub ticker: String,
    #[serde(flatten)] // flattens the Operation enum into the parent JSON object
    pub operation: Operation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")] // e.g., "action": "BUY"
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
        amount: Decimal,   // Net amount received
        tax_paid: Decimal, // Withholding tax
    },
    #[serde(rename = "CAPRETURN")]
    CapReturn { amount: Decimal, expenses: Decimal },
    Split {
        ratio: Decimal, // e.g. 2.0 for 2-for-1 split
    },
    Unsplit {
        ratio: Decimal, // e.g. 0.5 for 1-for-2 reverse split
    },
}
```

### 2. Holding (Section 104 Pool)

Represents the state of a Section 104 holding for a specific asset.

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct Section104Holding {
    pub ticker: String,
    pub quantity: Decimal,
    pub total_cost: Decimal, // Allowable cost
}
```

### 3. Match (Disposal Report)

Represents a calculated gain/loss event, linking a disposal to its acquisition source(s).

```rust
#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub date: NaiveDate,
    pub ticker: String,
    pub quantity: Decimal,
    pub proceeds: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub rule: MatchRule,
}

#[derive(Debug, Clone, Serialize)]
pub enum MatchRule {
    SameDay,
    BedAndBreakfast,
    Section104,
}
```

### 4. TaxReport

The final output aggregation.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct TaxReport {
    pub tax_year: String, // e.g., "2025/2026"
    pub matches: Vec<Match>,
    pub total_gain: Decimal,
    pub total_loss: Decimal,
    pub net_gain: Decimal,
    pub holdings: Vec<Section104Holding>, // Remaining holdings at end of report
}
```

## Validation Rules

1. **Decimal Precision**: All money/quantity fields use `Decimal`.
2. **Positive Values**: Amount and Price must be non-negative (enforced at parser or validation layer).
3. **Negative Holdings**: A `Sell` action cannot reduce `Section104Holding.quantity` below zero (unless shorting enabled - currently disabled).
4. **Expenses**: Expenses are deducted from disposal proceeds or added to acquisition costs (needs explicit handling in `calculator.rs`).

## State Transitions (Calculator Logic)

1. **Ingest**: Parse all transactions.
2. **Sort**: Chronological order (DATE ASC).
3. **Process**: Iterate through transactions:
   - **BUY**: Add to "Pending Buys" (for Same Day/B&B checks) or commit to S104 Pool.
   - **SELL**:
     1. Check **Same Day**: Match against Buys on `date == sell.date`.
     2. Check **Bed & Breakfast**: Match against Buys on `sell.date < buy.date <= sell.date + 30 days`.
     3. Check **Section 104**: Match remaining quantity against the Pool.
   - **SPLIT/UNSPLIT**: Adjust Pool quantity, keep Total Cost same.
   - **DIVIDEND**: Record income (separate from CGT, but tracked).
   - **RETURN_CAPITAL**: Reduce Pool Total Cost.
