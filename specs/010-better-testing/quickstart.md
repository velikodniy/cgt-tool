# Quickstart: Better Testing Coverage

## Prerequisites

- Rust toolchain installed (matching project's stable toolchain)
- No external services required

## Run tests

- Run full suite: `cargo test`
- Run data-driven matching tests: `cargo test test_data_driven_matching`
- Run with output: `cargo test test_data_driven_matching -- --nocapture`

## Fixture locations

- Inputs: `tests/inputs/`
- Expected text outputs: `tests/plain/`
- Expected JSON outputs: `tests/json/`

## New fixtures added by this feature

### User Story 1 - 2024/25 Rate Split

| Fixture             | Purpose                                           |
| ------------------- | ------------------------------------------------- |
| `RateSplit2024.cgt` | Tests disposals on 29 Oct vs 30 Oct 2024 boundary |

### User Story 2 - Pools and Gains

| Fixture                         | Purpose                                                     |
| ------------------------------- | ----------------------------------------------------------- |
| `AccumulationDividend.cgt`      | Dividend adjusts pool only for shares held on dividend date |
| `DividendAfterFullDisposal.cgt` | Dividend when holdings are zero (no error, no pool change)  |
| `CapReturnEqualisation.cgt`     | CAPRETURN reduces pool cost by lump sum                     |
| `ExpensesRounding.cgt`          | Expenses/stamp duty treatment and precision handling        |

### User Story 3 - Reporting Clarity

| Fixture                 | Purpose                                                |
| ----------------------- | ------------------------------------------------------ |
| `BnBReportQuantity.cgt` | B&B reports matched quantity (30), not full pool (100) |

### Edge Cases

| Fixture                  | Purpose                                        |
| ------------------------ | ---------------------------------------------- |
| `WhitespaceDividend.cgt` | Parser handles tabs and extra spaces correctly |

## Notes

- This feature is test-only; no new APIs or storage are introduced.
- Each fixture file includes detailed verification notes explaining the HMRC rules and manual calculations.
- The DSL does not support currency codes; all values are implicitly GBP. Non-numeric price values would produce a parse error.
