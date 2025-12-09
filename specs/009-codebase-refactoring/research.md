# Research: Codebase Quality Refactoring

**Feature**: 009-codebase-refactoring
**Date**: 2025-12-09

## Research Areas

### 1. Template Engine for Plain Text Formatter

**Decision**: Use `minijinja` for plain text templating

**Rationale**:

- Lightweight and fast (important for CLI tools)
- Jinja2-compatible syntax (widely understood)
- No runtime dependencies beyond the crate
- Excellent error messages for template debugging
- Active maintenance and Rust-idiomatic API
- Supports custom filters for formatting functions

**Alternatives Considered**:

- `tera`: More feature-rich but heavier; Jinja2-like syntax; good choice but larger dependency footprint
- `handlebars-rust`: Logic-less templates may be too restrictive for report formatting
- `askama`: Compile-time templates are fast but require recompilation for template changes (less flexible for users)

### 2. TOML Parsing for Exemption Configuration

**Decision**: Use `toml` crate with `serde` for deserialization

**Rationale**:

- Already have `serde` as a dependency
- `toml` crate is the de-facto standard for Rust TOML parsing
- Excellent error messages for malformed TOML
- Supports embedding via `include_str!` for compile-time defaults

**Alternatives Considered**:

- `toml_edit`: Preserves formatting but overkill for read-only config
- Manual parsing: Error-prone and unnecessary given excellent crate ecosystem

### 3. Currency Formatting Approach

**Decision**: Implement shared `format_currency` in `cgt-core::formatting` module

**Rationale**:

- `rust_decimal` doesn't provide locale-aware formatting out of the box
- UK currency formatting has specific rules (£ symbol, comma thousands separator, sign before symbol for negatives)
- A single shared implementation ensures consistency across all formatters
- Custom implementation is simple (~20 lines) and avoids heavy i18n dependencies

**Alternatives Considered**:

- `num-format` crate: Adds dependency for relatively simple formatting
- Delegate to Typst for PDF (already works) but need equivalent for plain text

**Formatting Convention Decided**:

- Negative values: `-£1,234` (sign before currency symbol)
- Thousands separator: comma (`,`)
- Decimal places: floor to whole pounds for display (consistent with HMRC)

### 4. Parser Error Enhancement Strategy

**Decision**: Use `pest`'s built-in error reporting with custom error transformation

**Rationale**:

- `pest` already captures line/column information in parse errors
- Custom error messages can wrap `pest::error::Error` with domain-specific suggestions
- No need to switch parsing libraries; enhance existing infrastructure

**Implementation Approach**:

1. Create `ParseErrorContext` struct with line, column, found token, expected tokens
2. Map common error patterns to helpful suggestions (e.g., invalid transaction type → list valid types)
3. Format errors with code snippets showing the problematic line

**Alternatives Considered**:

- `chumsky`: More powerful error recovery but would require rewriting entire parser
- `nom`: Good errors but different parsing paradigm; migration cost too high
- `ariadne`: Excellent error display but overkill for CLI; could add later for enhanced UX

### 5. Matcher Architecture Pattern

**Decision**: Extract matching logic into a `Matcher` struct with a pipeline of passes

**Rationale**:

- Current `calculate()` function is 600+ lines mixing concerns
- Each UK CGT matching rule (Same Day, B&B, Section 104) is conceptually independent
- Pipeline pattern allows unit testing each rule in isolation
- Clear data flow: transactions → preprocessed → same-day matches → B&B matches → Section 104 matches → report

**Design**:

```
Matcher {
    acquisition_ledger: AcquisitionLedger,  // Tracks remaining shares per acquisition
}

impl Matcher {
    fn preprocess(&mut self, transactions: &[Transaction]) -> Vec<ProcessedTransaction>
    fn apply_corporate_actions(&mut self, transactions: &mut [ProcessedTransaction])
    fn match_same_day(&mut self, transactions: &[ProcessedTransaction]) -> Vec<Match>
    fn match_bed_and_breakfast(&mut self, transactions: &[ProcessedTransaction]) -> Vec<Match>
    fn match_section_104(&mut self, transactions: &[ProcessedTransaction], pools: &mut HashMap<String, Section104Holding>) -> Vec<Match>
}
```

**Alternatives Considered**:

- Trait-based strategy pattern: Overengineered for 3 fixed rules
- Keeping monolithic function: Violates "deep modules" principle

### 6. Acquisition Ledger Design

**Decision**: Implement `AcquisitionLedger` as a per-ticker FIFO queue with cost adjustments

**Rationale**:

- Current O(n²) loops for CAPRETURN/DIVIDEND can be replaced with O(n) ledger updates
- Ledger tracks remaining shares and adjusted cost basis per acquisition lot
- FIFO ordering matches UK CGT rules for corporate action apportionment

**Data Structure**:

```rust
struct AcquisitionLot {
    date: NaiveDate,
    remaining_quantity: Decimal,
    original_quantity: Decimal,
    unit_cost: Decimal,
    cost_adjustments: Decimal, // From CAPRETURN (negative) / DIVIDEND (positive)
}

struct AcquisitionLedger {
    lots: HashMap<String, VecDeque<AcquisitionLot>>, // ticker -> FIFO queue
}
```

**Alternatives Considered**:

- Recompute on demand: Current approach; too slow for large transaction sets
- Single pool per ticker: Loses lot-level granularity needed for corporate actions

### 7. Input Validation Strategy

**Decision**: Implement a `validate()` function that runs before `calculate()`

**Rationale**:

- Fail-fast principle: catch issues before complex calculation begins
- Better error messages: validation errors are more actionable than calculation errors
- Separation of concerns: validation logic doesn't belong in calculator

**Validations to Implement**:

1. Zero-quantity disposals → Error
2. Division by zero guards (quantity == 0 in cost calculations) → Error
3. Sell before any buy for ticker → Warning
4. Overflow detection in currency multiplication → Error

**Alternatives Considered**:

- Inline validation during calculation: Mixes concerns, harder to test
- Type-level validation (newtypes): Deferred to future feature per spec

### 8. Exemption Data Source

**Decision**: Embed TOML file at compile time, support optional runtime override

**Rationale**:

- Embedded defaults ensure tool works without external files
- TOML override enables users to add new tax years without recompilation
- HMRC publishes exemption values publicly; data is straightforward to maintain

**Data Format** (`exemptions.toml`):

```toml
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

**Override Location**: `~/.config/cgt-tool/exemptions.toml` or `./exemptions.toml` (checked in order)

## Dependencies Summary

| Dependency  | Purpose                     | Version | Notes                      |
| ----------- | --------------------------- | ------- | -------------------------- |
| `toml`      | TOML parsing for exemptions | latest  | Add to cgt-core            |
| `minijinja` | Plain text templating       | latest  | Add to cgt-formatter-plain |
| `dirs`      | XDG config path resolution  | latest  | Add to cgt-core (optional) |

## Risks and Mitigations

| Risk                                                | Impact | Mitigation                                                                     |
| --------------------------------------------------- | ------ | ------------------------------------------------------------------------------ |
| Template engine adds complexity                     | Medium | Start with simple templates; minijinja is lightweight                          |
| Matcher refactoring breaks calculations             | High   | Comprehensive test coverage; run all existing tests after each pass extraction |
| Parser error changes affect existing error handling | Low    | Errors are enhanced, not changed; backward compatible                          |
| TOML override file could have invalid data          | Low    | Validate all exemption values are positive; clear error messages               |

## Open Questions (Resolved)

All research questions resolved. Ready for Phase 1 design.
