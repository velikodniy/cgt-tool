# Implementation Plan: Multi-Ticker Support

**Branch**: `006-multi-ticker` | **Date**: 2025-12-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-multi-ticker/spec.md`

## Summary

Enable the CGT calculator to handle multiple ticker symbols in a single transaction file. Currently, the calculator uses a single `Option<Section104Holding>` pool, which breaks when transactions contain multiple tickers. The solution follows a "split-process-merge" pattern: group transactions by ticker, process each group independently through the existing 3-pass matching logic, then merge results.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: pest (parsing), chrono (dates), rust_decimal (numbers), thiserror (errors)
**Storage**: File-based (.cgt input, JSON output)
**Testing**: cargo test (data-driven tests with JSON expected outputs)
**Target Platform**: macOS/Linux CLI
**Project Type**: Single (workspace with cgt-core library + cgt-cli binary)
**Performance Goals**: N/A (batch processing, small datasets)
**Constraints**: Strict Clippy lints (deny unwrap, expect, panic)
**Scale/Scope**: Typical portfolio: 10-50 tickers, 100-1000 transactions per year

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle                         | Status  | Notes                                                                       |
| --------------------------------- | ------- | --------------------------------------------------------------------------- |
| I. Deep Modules & Simplicity      | ✅ PASS | Split-process-merge simplifies code by reusing existing single-ticker logic |
| II. Safety & Robustness           | ✅ PASS | Strict typing, explicit error handling maintained                           |
| III. Modern Testing Standards     | ✅ PASS | Existing tests preserved, new multi-ticker tests with manual calculations   |
| IV. User Experience Consistency   | ✅ PASS | CLI interface unchanged, output format compatible                           |
| V. Performance & Efficiency       | ✅ PASS | No performance regression expected                                          |
| VI. Domain Mastery & Verification | ✅ PASS | Manual calculations required for all test cases                             |

## Project Structure

### Documentation (this feature)

```text
specs/006-multi-ticker/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── calculator.rs   # Main changes: split-process-merge refactor
│   │   ├── models.rs       # No changes needed
│   │   ├── parser.rs       # Add ticker uppercase normalization
│   │   └── error.rs        # No changes needed
│   └── tests/
│       └── matching_tests.rs
└── cgt-cli/
    └── src/main.rs         # No changes needed

tests/data/
├── *.cgt                   # Existing single-ticker tests
├── *.json                  # Expected outputs
├── MultiTicker*.cgt        # NEW: Multi-ticker test files
└── MultiTicker*.json       # NEW: Expected outputs
```

**Structure Decision**: Existing Rust workspace structure with two crates. Changes isolated to cgt-core (calculator.rs refactor, parser.rs ticker normalization). New test files added to tests/data/.

## Implementation Approach

### Phase 1: Parser Change (FR-009)

Normalize ticker symbols to uppercase in `parser.rs` during parsing. This ensures "aapl" and "AAPL" are treated as the same ticker.

### Phase 2: Calculator Refactor (FR-001 to FR-008)

Refactor `calculate()` function using split-process-merge pattern:

1. **Split**: Group transactions by ticker using `HashMap<String, Vec<Transaction>>`
2. **Process**: For each ticker group, run existing 3-pass matching (Same Day → B&B → Section 104)
3. **Merge**: Combine all ticker results into single `TaxReport`

Key insight: The existing matching logic already works correctly for a single ticker. By grouping first, we isolate each ticker's transactions and avoid cross-ticker matching bugs.

### Phase 3: Testing (SC-001 to SC-004)

1. Verify all existing tests pass (backward compatibility)
2. Add minimum 3 multi-ticker test cases with manual calculations:
   - Basic multi-ticker (different dates, Section 104 only)
   - Same day with multiple tickers (no cross-matching)
   - B&B with multiple tickers (no cross-matching)
