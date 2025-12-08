# Data Model: Multi-Ticker Support

**Feature**: 006-multi-ticker
**Date**: 2025-12-08

## Overview

This feature requires minimal data model changes. The existing models already support multiple tickers in the output (`Vec<Section104Holding>`, `Vec<Disposal>`). The main change is internal to the calculator.

## Existing Entities (No Changes Required)

### Transaction

```
Transaction {
    date: NaiveDate
    ticker: String        # Now normalized to uppercase during parsing
    operation: Operation  # Buy/Sell/Split/Unsplit/CapReturn/Dividend
}
```

### Section104Holding

```
Section104Holding {
    ticker: String
    quantity: Decimal
    total_cost: Decimal
}
```

### Disposal

```
Disposal {
    date: NaiveDate
    ticker: String
    quantity: Decimal
    proceeds: Decimal
    matches: Vec<Match>
}
```

### TaxReport

```
TaxReport {
    tax_years: Vec<TaxYearSummary>
    holdings: Vec<Section104Holding>  # Already supports multiple tickers
}
```

## Internal Data Structures (Calculator)

### Current (Single Ticker)

```rust
let mut pool: Option<Section104Holding> = None;
```

### New (Multi-Ticker)

```rust
let mut pools: HashMap<String, Section104Holding> = HashMap::new();
```

## Data Flow

```
Input (.cgt file)
    │
    ▼
Parser (normalize ticker to uppercase)
    │
    ▼
Vec<Transaction> (mixed tickers)
    │
    ▼
Group by ticker: HashMap<String, Vec<Transaction>>
    │
    ▼
┌─────────────────────────────────────────┐
│  For each ticker:                       │
│    ├─ Same Day matching                 │
│    ├─ B&B matching                      │
│    └─ Section 104 pooling               │
│                                         │
│  Result: (Vec<InternalMatch>, Pool)     │
└─────────────────────────────────────────┘
    │
    ▼
Merge: Combine disposals, collect pools
    │
    ▼
TaxReport (multi-ticker output)
```

## Validation Rules

| Rule              | Entity      | Description                                                  |
| ----------------- | ----------- | ------------------------------------------------------------ |
| Ticker format     | Transaction | Any non-empty string, normalized to uppercase                |
| Ticker isolation  | Calculator  | Each ticker processed independently                          |
| Pool uniqueness   | Holdings    | One pool per ticker (HashMap key)                            |
| Disposal grouping | Output      | Disposals sorted by date, may have multiple tickers per date |

## State Transitions

### Section 104 Pool (per ticker)

```
[Empty] ──BUY──▶ [Holding: qty, cost]
                        │
                   ┌────┴────┐
                   │         │
                  BUY       SELL
                   │         │
                   ▼         ▼
           [+qty, +cost] [-qty, -cost]
                   │         │
                   └────┬────┘
                        │
                      SPLIT
                        │
                        ▼
                   [qty × ratio]
                        │
                      UNSPLIT
                        │
                        ▼
                   [qty ÷ ratio]
```

Each ticker maintains its own independent state machine.
