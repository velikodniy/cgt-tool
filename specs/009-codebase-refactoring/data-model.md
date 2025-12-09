# Data Model: Codebase Quality Refactoring

**Feature**: 009-codebase-refactoring
**Date**: 2025-12-09

## Overview

This document describes new and modified data structures for the refactoring. Existing models in `cgt-core::models` are preserved; this documents additions and internal structures.

## New Entities

### 1. AcquisitionLot

Represents a single acquisition event with cost adjustments from corporate actions.

```rust
/// A single acquisition lot tracking remaining shares and cost basis
pub struct AcquisitionLot {
    /// Date of the original acquisition
    pub date: NaiveDate,
    /// Shares remaining (decremented by matching)
    pub remaining_quantity: Decimal,
    /// Original shares acquired
    pub original_quantity: Decimal,
    /// Original cost per share (price + expenses/quantity)
    pub unit_cost: Decimal,
    /// Cumulative cost adjustments from corporate actions
    /// Positive for DIVIDEND, negative for CAPRETURN
    pub cost_adjustment: Decimal,
}
```

**Invariants**:

- `remaining_quantity >= 0`
- `remaining_quantity <= original_quantity`
- `original_quantity > 0`

**State Transitions**:

- Created: When BUY transaction processed
- Updated: When CAPRETURN/DIVIDEND applies cost adjustment
- Consumed: When shares matched to disposal (remaining_quantity decremented)

### 2. AcquisitionLedger

Per-ticker FIFO queue of acquisition lots for efficient corporate action processing.

```rust
/// Tracks all acquisition lots by ticker in FIFO order
pub struct AcquisitionLedger {
    /// Map of ticker -> FIFO queue of acquisition lots
    lots: HashMap<String, VecDeque<AcquisitionLot>>,
}

impl AcquisitionLedger {
    /// Add a new acquisition lot for a ticker
    pub fn add_acquisition(&mut self, ticker: &str, lot: AcquisitionLot);

    /// Get remaining shares for a ticker at a given date
    pub fn remaining_shares(&self, ticker: &str, as_of: NaiveDate) -> Decimal;

    /// Apply cost adjustment to lots held before a date (for CAPRETURN/DIVIDEND)
    pub fn apply_cost_adjustment(
        &mut self,
        ticker: &str,
        before_date: NaiveDate,
        adjustment_per_share: Decimal,
    );

    /// Consume shares FIFO, returning matched lots with their costs
    pub fn consume_shares(
        &mut self,
        ticker: &str,
        quantity: Decimal,
    ) -> Result<Vec<ConsumedLot>, CgtError>;
}
```

### 3. ConsumedLot

Result of consuming shares from the acquisition ledger.

```rust
/// A portion of an acquisition lot consumed by a disposal
pub struct ConsumedLot {
    pub acquisition_date: NaiveDate,
    pub quantity: Decimal,
    pub cost: Decimal, // Includes cost adjustments
}
```

### 4. ExemptionConfig

Configuration for CGT annual exemptions.

```rust
/// Tax exemption configuration loaded from TOML
#[derive(Debug, Deserialize)]
pub struct ExemptionConfig {
    /// Map of tax year start (e.g., 2023) to exemption amount
    pub exemptions: HashMap<u16, Decimal>,
}

impl ExemptionConfig {
    /// Load embedded defaults
    pub fn embedded() -> Self;

    /// Load from file, falling back to embedded
    pub fn load_with_overrides(override_path: Option<&Path>) -> Result<Self, CgtError>;

    /// Get exemption for a tax year
    pub fn get(&self, year: u16) -> Result<Decimal, CgtError>;
}
```

### 5. ValidationResult

Result of input validation pass.

```rust
/// Validation result with errors and warnings
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

pub struct ValidationError {
    pub message: String,
    pub location: Option<TransactionLocation>,
}

pub struct ValidationWarning {
    pub message: String,
    pub location: Option<TransactionLocation>,
}

pub struct TransactionLocation {
    pub index: usize,
    pub date: NaiveDate,
    pub ticker: String,
}
```

### 6. ParseErrorContext

Enhanced parser error with context and suggestions.

```rust
/// Rich parser error with location and suggestions
pub struct ParseErrorContext {
    pub line: usize,
    pub column: usize,
    pub found: String,
    pub expected: Vec<String>,
    pub suggestion: Option<String>,
    pub source_line: Option<String>,
}

impl std::fmt::Display for ParseErrorContext {
    // Formats as:
    // Error at line 5, column 12: unexpected 'BYYY'
    //   5 | 2023-04-06 BYYY AAPL 100 150.00
    //                 ^^^^
    // Expected: BUY, SELL, SPLIT, UNSPLIT, DIVIDEND, CAPRETURN
    // Hint: Did you mean 'BUY'?
}
```

### 7. FormattingPolicy

Shared formatting rules for currency, dates, and numbers.

```rust
/// Formatting policy for consistent output across formatters
pub struct FormattingPolicy {
    pub currency_symbol: &'static str,   // "£"
    pub thousands_separator: char,       // ','
    pub negative_format: NegativeFormat, // SignBeforeSymbol
    pub date_format: &'static str,       // "%d/%m/%Y"
    pub tax_year_format: TaxYearFormat,  // "2023/24"
}

pub enum NegativeFormat {
    SignBeforeSymbol, // -£100
    SignAfterSymbol,  // £-100
    Parentheses,      // (£100)
}

pub enum TaxYearFormat {
    SlashShort, // 2023/24
    SlashFull,  // 2023/2024
    Hyphen,     // 2023-24
}

impl FormattingPolicy {
    /// UK CGT default policy
    pub fn uk_default() -> Self;

    pub fn format_currency(&self, value: Decimal) -> String;
    pub fn format_date(&self, date: NaiveDate) -> String;
    pub fn format_tax_year(&self, start_year: u16) -> String;
    pub fn format_decimal(&self, value: Decimal) -> String;
}
```

## Modified Entities

### Matcher (replaces inline calculator logic)

```rust
/// Orchestrates CGT matching rules with isolated passes
pub struct Matcher {
    ledger: AcquisitionLedger,
    pools: HashMap<String, Section104Holding>,
}

impl Matcher {
    pub fn new() -> Self;

    /// Preprocess transactions: sort, merge same-day, build ledger
    pub fn preprocess(&mut self, transactions: Vec<Transaction>) -> Vec<ProcessedTransaction>;

    /// Apply CAPRETURN/DIVIDEND cost adjustments via ledger
    pub fn apply_corporate_actions(&mut self, transactions: &[ProcessedTransaction]);

    /// Pass 1: Match same-day acquisitions
    pub fn match_same_day(&mut self, transactions: &[ProcessedTransaction]) -> Vec<InternalMatch>;

    /// Pass 2: Match bed-and-breakfast (within 30 days)
    pub fn match_bed_and_breakfast(
        &mut self,
        transactions: &[ProcessedTransaction],
        consumed: &mut [Decimal],
    ) -> Vec<InternalMatch>;

    /// Pass 3: Match against Section 104 pool
    pub fn match_section_104(
        &mut self,
        transactions: &[ProcessedTransaction],
        consumed: &[Decimal],
    ) -> Result<Vec<InternalMatch>, CgtError>;

    /// Get final holdings after all matching
    pub fn holdings(&self) -> Vec<Section104Holding>;
}
```

## Relationships

```
Transaction (existing)
    │
    ▼
Matcher.preprocess()
    │
    ▼
ProcessedTransaction ──► AcquisitionLedger
    │                         │
    │                         ▼
    │                    AcquisitionLot
    │
    ▼
Matcher.match_*() passes
    │
    ▼
InternalMatch ──► grouped into Disposal (existing)
    │
    ▼
TaxReport (existing)
```

## Data Files

### exemptions.toml (embedded)

```toml
# UK Capital Gains Tax Annual Exempt Amounts
# Source: HMRC
# Tax year is identified by start year (e.g., 2023 = 2023/24 tax year)

[exemptions]
2014 = 11000
2015 = 11100
2016 = 11100
2017 = 11300
2018 = 11700
2019 = 12000
2020 = 12300
2021 = 12300
2022 = 12300
2023 = 6000
2024 = 3000
```

### Override file (~/.config/cgt-tool/exemptions.toml)

Same format as embedded; values override/extend embedded defaults.
