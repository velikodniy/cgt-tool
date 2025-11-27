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

### 4. DIVIDEND and CAPRETURN Format: TOTAL vs Per-Share

- **Decision**: Use `TOTAL` keyword for the total value, not per-share amounts.
- **Rationale**: The original `cgtcalc` format uses total values: `CAPRETURN DD/MM/YYYY TICKER 15 50` means 15 shares for £50 total, not £50 per share. This is confirmed by output: "CAPITAL RETURN on 15 for £50".
- **Impact**: DSL format is: `YYYY-MM-DD CAPRETURN TICKER 15 TOTAL 50 EXPENSES 0`

### 5. Capital Returns and Dividends Processing Order

- **Decision**: Implement preprocessing step to apply all CAPRETURN and DIVIDEND events before running the matching algorithm, following the original `cgtcalc` implementation.
- **Rationale**: HMRC rules for capital returns (small distributions under TCGA92/S122(2)) require reducing the allowable cost of shares. The original `cgtcalc` implementation interprets these rules as applying to all acquisitions held at the time of the event, and using those adjusted costs for calculating gains on all sales within the same tax year—even if the sale happened chronologically before the capital return.
- **Legal Basis**:
  - [HMRC Capital Gains Manual CG57835](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57835) - Small capital distributions: "The amount of the distribution is deducted from the allowable cost of the shares"
  - [HMRC Capital Gains Manual CG57800P](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57800p) - Capital distributions overview
  - [HS284 Shares and Capital Gains Tax (2021)](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet/hs284-shares-and-capital-gains-tax-2021) - Section 104 holding and matching order
- **Implementation**: The calculator must:
  1. First pass: Apply all CAPRETURN events to reduce acquisition costs
  2. Second pass: Apply all DIVIDEND events to increase acquisition costs
  3. Third pass: Run matching algorithm (Same Day, Bed & Breakfast, Section 104) with adjusted costs
- **Impact**: Requires significant refactoring of `calculator.rs` to track individual acquisitions and apply preprocessing adjustments before matching.

## Unknowns Resolved

- The approach for incorporating new keywords into the `pest` grammar is clear: insert literals within the `buy_sell_args`, `dividend_args`, etc. rules.
- Handling optional `EXPENSES` in `BUY/SELL` is also clear: `(SEP ~ "EXPENSES" ~ SEP ~ expenses)?` in grammar and `if let Some` in parser.
- The strategy for sorting `tests/data/*.cgt` files will involve either re-downloading and re-converting the data, or sorting them in-place, then re-generating expected JSONs. Since `cgtcalc` provides sorted inputs, and the calculator sorts, we must ensure test inputs are truly representative. This will be a manual step in tests.
- Verification against original `cgtcalc` outputs will involve careful comparison, potentially with manual intervention to resolve rounding discrepancies.
- DIVIDEND and CAPRETURN use TOTAL values, not per-share amounts, based on analysis of original `cgtcalc` inputs and outputs.
- Capital returns and dividends must be preprocessed to adjust acquisition costs before running the matching algorithm, as per HMRC guidance and the original `cgtcalc` implementation.
