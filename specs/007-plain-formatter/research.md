# Research: Plain Text Report Formatter

**Feature**: 007-plain-formatter
**Date**: 2025-12-08

## Output Format Analysis

### Decision: Match cgtcalc format exactly

**Rationale**: The cgtcalc tool by Matt Galloway is an established reference implementation. Using the same format ensures compatibility and familiarity for users migrating from or comparing with cgtcalc.

**Alternatives considered**:

- Custom format: Rejected - no benefit, loses compatibility
- HMRC-specific format: Rejected - no official standard exists

### Format Structure (from cgtcalc analysis)

```text
# SUMMARY

Tax year    Gain   Proceeds   Exemption   Loss carry   Taxable gain   Tax (basic)   Tax (higher)
================================================================================================
2018/2019   £-20   £46        £11700      £20          £0             £0            £0


# TAX YEAR DETAILS

## TAX YEAR 2018/2019

0 gains with total of 0.
1 losses with total of 20.

1) SOLD 10 of GB00B41YBW71 on 28/08/2018 for LOSS of £20
Matches with:
  - SAME DAY: 10 bought on 28/08/2018 at £4.1565
Calculation: (10 * 4.6702 - 12.5) - ( (10 * 4.1565 + 12.5) ) = -20


# TAX RETURN INFORMATION

2018/2019: Disposals = 1, proceeds = 46, allowable costs = 66, total gains = 0, total losses = 20


# HOLDINGS

NONE

# TRANSACTIONS

28/08/2018 SOLD 10 of GB00B41YBW71 at £4.6702 with £12.5 expenses
28/08/2018 BOUGHT 10 of GB00B41YBW71 at £4.1565 with £12.5 expenses


# ASSET EVENTS

NONE
```

## Formatter Crate Architecture

### Decision: Separate crate with single public function

**Rationale**: Follows the spec requirement (FR-009) for extensible architecture. A simple API makes it easy to add more formatters later.

**Interface**:

```rust
pub fn format(report: &TaxReport, transactions: &[Transaction]) -> String
```

**Alternatives considered**:

- Trait-based formatter: Overkill for current scope, but could refactor later
- Builder pattern: Unnecessary complexity for single output format

## Data Requirements

### Decision: Formatter needs both TaxReport and original Transactions

**Rationale**: The TRANSACTIONS section requires the original transaction data (buy/sell with prices and expenses), which isn't stored in TaxReport. The TaxReport only contains disposals and matches.

**Data flow**:

1. CLI parses .cgt file → `Vec<Transaction>`
2. CLI calculates → `TaxReport`
3. CLI passes both to formatter → formatted String

## Number Formatting

### Decision: Use £ symbol, round to nearest integer for display

**Rationale**: Matches cgtcalc behavior. Currency amounts display as `£46`, not `£46.00`. Fractional amounts like prices show full precision (e.g., `£4.1565`).

**Implementation notes**:

- Currency totals: round to integer
- Per-share prices: show full decimal precision
- Quantities: show full decimal precision (for fractional shares)

## CLI Integration

### Decision: Add `--format` argument with enum values

**Rationale**: Clean, type-safe approach using clap's enum support.

```rust
#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Plain,
    Json,
}
```

**Default**: Plain (as per FR-008)

## Test Data

### Decision: Add .txt expected output files alongside existing .json files

**Rationale**: Same pattern as existing tests - compare generated output against expected files.

**Files to create**:

- Simple.txt, GainsAndLosses.txt, HMRCExample1.txt, MultipleMatches.txt (from cgtcalc)
- Plus .txt versions for all existing test .cgt files

## UK Tax Exemption Data

### Decision: Hardcode annual exemption values by year

**Rationale**: The SUMMARY table includes "Exemption" column. Values are set by HMRC annually.

**Known values**:

- 2018/2019: £11,700
- 2019/2020: £12,000
- 2020/2021: £12,300
- 2023/2024: £6,000
- 2024/2025: £3,000

**Note**: Could be moved to configuration file later if needed.
