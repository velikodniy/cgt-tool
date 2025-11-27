# Research: DSL Enhancements

**Feature**: `002-dsl-enhancements`
**Date**: 2025-11-27

## Decisions & Rationale

### 1. Grammar Updates for Readability Keywords

- **Decision**: Update `parser.pest` to include explicit keywords (`TAX`, `EXPENSES`, `SPLIT`) within command arguments.
- **Rationale**: Directly addresses FR-001, FR-002, FR-003 for enhanced DSL readability. `pest` allows literal matching of these keywords.
- **Impact**: Requires corresponding updates to `parser.rs` to correctly parse the new argument sequence.

### 2. Flexible Whitespace and Comments Support

- **Decision**: Implement `WHITESPACE` and `COMMENT` rules in `parser.pest`.
- **Rationale**: Addresses the need for more flexible input (`FR-002` implicitly, as well as general user experience). `pest` automatically consumes `WHITESPACE` between rules and allows defining `COMMENT` rules.
- **Impact**: Simplifies parser implementation by offloading whitespace/comment handling to `pest`.

### 3. Test Data Re-validation Strategy

- **Decision**: Implement a process to re-download original `cgtcalc` test data and re-convert it to the new DSL format and expected JSON output. This includes sorting transactions in `.cgt` files and meticulously comparing against original `cgtcalc` outputs.
- **Rationale**: Directly addresses FR-006 (sort transactions), FR-007 (re-validate outputs), and FR-008 (trust original outputs). Ensures the robust test suite.
- **Impact**: Requires a temporary script or manual process to download and convert original test data. Potential for further refinement of `matching_tests.rs` to handle more precise comparisons or to re-generate expected JSONs based on downloaded data.

## Unknowns Resolved

- The approach for incorporating new keywords into the `pest` grammar is clear: insert literals within the `buy_sell_args`, `dividend_args`, etc. rules.
- Handling optional `EXPENSES` in `BUY/SELL` is also clear: `(SEP ~ "EXPENSES" ~ SEP ~ expenses)?` in grammar and `if let Some` in parser.
- The strategy for sorting `tests/data/*.cgt` files will involve either re-downloading and re-converting the data, or sorting them in-place, then re-generating expected JSONs. Since `cgtcalc` provides sorted inputs, and the calculator sorts, we must ensure test inputs are truly representative. This will be a manual step in tests.
- Verification against original `cgtcalc` outputs will involve careful comparison, potentially with manual intervention to resolve rounding discrepancies.
