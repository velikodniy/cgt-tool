# Quickstart: Multi-Ticker Support

**Feature**: 006-multi-ticker
**Date**: 2025-12-08

## Prerequisites

- Rust toolchain (2024 edition)
- Understanding of UK CGT matching rules (see TAX_RULES.md)
- Familiarity with existing calculator.rs structure

## Quick Overview

This feature enables the CGT calculator to process transactions for multiple ticker symbols in a single file. The implementation follows a "split-process-merge" pattern.

## Key Changes

### 1. Parser: Ticker Normalization

**File**: `crates/cgt-core/src/parser.rs`

Add uppercase normalization when parsing ticker:

```rust
// In parse_buy_sell or wherever ticker is extracted:
let ticker = ticker_str.to_uppercase();
```

### 2. Calculator: Split-Process-Merge

**File**: `crates/cgt-core/src/calculator.rs`

#### Step 1: Group transactions by ticker

```rust
use std::collections::HashMap;

// After sorting and merging same-day transactions:
let mut by_ticker: HashMap<String, Vec<(usize, &Transaction)>> = HashMap::new();
for (idx, tx) in transactions.iter().enumerate() {
    by_ticker.entry(tx.ticker.clone()).or_default().push((idx, tx));
}
```

#### Step 2: Process each ticker independently

Extract the existing 3-pass matching logic into a helper function:

```rust
fn calculate_single_ticker(
    transactions: &[Transaction],
    acquisition_trackers: &[Option<AcquisitionTracker>],
    consumed: &mut [Decimal],
    original_indices: &[usize],
) -> Result<(Vec<InternalMatch>, Option<Section104Holding>), CgtError> {
    // Existing Same Day, B&B, Section 104 logic here
    // All index references use original_indices mapping
}
```

#### Step 3: Merge results

```rust
let mut all_matches: Vec<InternalMatch> = Vec::new();
let mut pools: HashMap<String, Section104Holding> = HashMap::new();

for (ticker, ticker_txs) in by_ticker {
    let (matches, pool) = calculate_single_ticker(...)?;
    all_matches.extend(matches);
    if let Some(p) = pool {
        if p.quantity > Decimal::ZERO {
            pools.insert(ticker, p);
        }
    }
}

// Convert pools to holdings vector
let holdings: Vec<Section104Holding> = pools.into_values().collect();
```

### 3. Test Files

Create new test files in `tests/data/`:

#### MultiTickerBasic.cgt

```
# Test: MultiTickerBasic
# Purpose: Basic multi-ticker Section 104 pooling
# Rules Tested: Section 104
# Complexity: Simple
# Key Features: Two tickers with independent pools
# Expected Outcome: Each ticker has separate pool and gain/loss
#
# Verification Status: Verified
# Verified By: Manual calculation YYYY-MM-DD
# Verification Notes:
#   AAPL: Buy 10 @ £100 = £1000, Sell 5 @ £120 = £600
#         Cost: £500, Gain: £100
#   MSFT: Buy 20 @ £50 = £1000, Sell 10 @ £40 = £400
#         Cost: £500, Loss: -£100
#   Net: £0

2023-06-01 BUY AAPL 10 @ 100 EXPENSES 0
2023-06-01 BUY MSFT 20 @ 50 EXPENSES 0
2023-09-01 SELL AAPL 5 @ 120 EXPENSES 0
2023-09-01 SELL MSFT 10 @ 40 EXPENSES 0
```

## Testing

```bash
# Run all tests (should pass - backward compatible)
cargo test

# Run specific multi-ticker test
cargo test test_data_driven_matching -- --nocapture

# Check for clippy violations
cargo clippy
```

## Verification Checklist

- [ ] All existing tests pass without modification (SC-001)
- [ ] Ticker normalization works (FR-009): "aapl" → "AAPL"
- [ ] Multi-ticker pools are independent (FR-001)
- [ ] Same Day only matches same ticker (FR-002)
- [ ] B&B only matches same ticker (FR-003)
- [ ] Split/Unsplit only affects specified ticker (FR-004, FR-005)
- [ ] Holdings output shows all tickers (FR-006)
- [ ] Error on sell without prior acquisition (FR-007)
- [ ] Error on oversell (FR-008)
- [ ] Minimum 3 multi-ticker test cases with manual calculations (SC-002)

## Common Pitfalls

1. **Index mapping**: When processing a ticker subset, map back to original transaction indices for `consumed` array
2. **Empty pools**: Filter out tickers with zero quantity before output
3. **Date sorting**: Ensure disposals are sorted by date in final output (may have multiple tickers per date)
4. **Acquisition trackers**: These are indexed by original transaction position, not ticker-filtered position
