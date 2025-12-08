# Research: Multi-Ticker Support

**Feature**: 006-multi-ticker
**Date**: 2025-12-08

## Summary

This document consolidates research findings for the multi-ticker support feature. The existing codebase was analyzed to identify the minimal changes required.

## Research Findings

### 1. Current Calculator Architecture

**Decision**: Refactor using "split-process-merge" pattern rather than adding ticker filtering to each matching pass.

**Rationale**:

- The existing 3-pass matching logic (Same Day → B&B → Section 104) works correctly for single-ticker scenarios
- Adding ticker filters to each pass would increase complexity and risk introducing bugs
- Grouping by ticker first allows complete reuse of existing logic
- Cleaner separation of concerns: grouping is a preprocessing step, matching logic remains unchanged

**Alternatives Considered**:

1. **Add ticker filtering to each pass**: More invasive changes, higher bug risk, harder to test
2. **Create per-ticker calculator instances**: Unnecessary object overhead, complicates result merging
3. **Split-process-merge (chosen)**: Minimal code changes, maximal code reuse

### 2. Ticker Normalization Location

**Decision**: Normalize ticker to uppercase in `parser.rs` during transaction parsing.

**Rationale**:

- Single point of normalization (DRY principle)
- All downstream code receives already-normalized tickers
- No risk of case-sensitivity bugs in calculator or output
- Aligns with real-world exchange behavior (tickers are uppercase)

**Alternatives Considered**:

1. **Normalize in calculator**: Would need to normalize in multiple places (matching, pooling, output)
2. **Case-insensitive HashMap**: More complex, potential subtle bugs
3. **Parser normalization (chosen)**: Simple, single location, complete coverage

### 3. Data Structure for Multi-Ticker Pools

**Decision**: Use `HashMap<String, Section104Holding>` keyed by ticker symbol.

**Rationale**:

- Natural mapping: one pool per ticker
- O(1) lookup for any ticker
- Easy to convert to `Vec<Section104Holding>` for output
- Rust's HashMap provides the exact semantics needed

**Alternatives Considered**:

1. **Vec with linear search**: O(n) lookup, unnecessary complexity
2. **BTreeMap**: Ordered output, but ordering not required by spec
3. **HashMap (chosen)**: Simple, efficient, idiomatic Rust

### 4. Result Merging Strategy

**Decision**: Process each ticker independently, then concatenate disposals and holdings.

**Rationale**:

- Disposals are already grouped by (date, ticker) in existing code
- Holdings are per-ticker by definition
- Total gains/losses can be summed across all tickers
- No complex merging logic needed

**Implementation Approach**:

```
1. Group transactions by ticker
2. For each ticker:
   - Extract ticker's transactions
   - Run calculate_single_ticker() with existing logic
   - Collect disposals and pool state
3. Merge:
   - Concatenate all disposals, sort by date
   - Collect all non-empty pools as holdings
   - Sum gains/losses across all tickers
```

### 5. Test File Structure

**Decision**: Create new test files with "MultiTicker" prefix, following existing naming convention.

**Rationale**:

- Clear identification of multi-ticker tests
- Consistent with existing PascalCase naming (e.g., `SameDayMerge.cgt`)
- Separate from single-ticker tests for clarity

**Test Cases Required**:

1. `MultiTickerBasic.cgt` - Two tickers, Section 104 only
2. `MultiTickerSameDay.cgt` - Same day buys/sells of different tickers (no cross-match)
3. `MultiTickerBedAndBreakfast.cgt` - B&B scenario with multiple tickers (no cross-match)

## No Clarifications Needed

All technical decisions could be made based on:

- Existing codebase patterns
- Rust best practices
- UK CGT domain knowledge (already documented in TAX_RULES.md)
- Clarifications from spec (split-process-merge, uppercase normalization)
