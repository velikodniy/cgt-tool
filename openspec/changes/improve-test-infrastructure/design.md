# Design: Improve Test Infrastructure

## Context

The cgt-tool project has ~132 tests spread across inline modules and integration tests. Several source files contain inline `#[cfg(test)]` modules that increase file length (e.g., `cgt-mcp/src/server.rs` has 39 inline tests in an 1886-line file). Test coverage is unknown, and there's no systematic cross-validation against other UK CGT calculators.

### Current State

| File                                 | Inline Tests | Lines   |
| ------------------------------------ | ------------ | ------- |
| `cgt-mcp/src/server.rs`              | 39           | 1886    |
| `cgt-converter/src/schwab/awards.rs` | 26           | 672     |
| `cgt-converter/src/schwab/mod.rs`    | 22           | 625     |
| `cgt-format/src/lib.rs`              | 11           | 248     |
| `cgt-core/src/validation.rs`         | 9            | 506     |
| Other files                          | ~25          | Various |

### External Calculators for Cross-Validation

1. **KapJI/capital-gains-calculator** (Python)

   - Supports RAW format: `BUY/SELL DATE TICKER AMOUNT PRICE EXPENSES [CURRENCY]`
   - Run with: `uvx cgt-calc --year YYYY --raw-file input.txt`
   - Note: Date format is `DD/MM/YYYY`, our DSL uses `YYYY-MM-DD`

2. **mattjgalloway/cgtcalc** (Swift)

   - Format: `BUY DD/MM/YYYY TICKER AMOUNT PRICE EXPENSES`
   - Transaction order is reversed (newest first) in output but input order doesn't matter
   - Build: `swift build -c release`
   - Run: `.build/release/cgtcalc data.txt`

## Goals / Non-Goals

### Goals

- Reduce source file sizes by moving tests to separate files
- Enable test coverage measurement (target: >80% line coverage)
- Cross-validate calculations against established UK CGT calculators
- Add edge case tests for complex HMRC scenarios
- Create a realistic multi-year transaction fixture

### Non-Goals

- Achieving 100% coverage (diminishing returns)
- Fully automating cross-validation in CI (keep local/manual)
- Supporting every edge case from HMRC forums (prioritize common cases)

## Decisions

### Decision 1: Use cargo-llvm-cov for Coverage

**Rationale**: `cargo-llvm-cov` is the modern standard for Rust coverage, using LLVM's instrumentation. It supports HTML, lcov, and codecov formats. Unlike `tarpaulin`, it works reliably on all platforms.

**Installation**: `cargo install cargo-llvm-cov`

**Usage**:

```bash
cargo llvm-cov --html           # HTML report in target/llvm-cov/html/
```

Coverage reports are kept local only (no CI integration).

### Decision 2: Test File Organization

Move inline tests to dedicated files following this pattern:

```
crates/cgt-core/
├── src/
│   ├── lib.rs           # No #[cfg(test)] module
│   ├── validation.rs    # No #[cfg(test)] module
│   └── ...
└── tests/
    ├── lib_tests.rs      # Tests moved from lib.rs
    ├── validation_tests.rs
    ├── matching_tests.rs  # Already exists
    └── ...
```

Tests remain in the same crate, preserving access to `pub(crate)` items.

### Decision 3: Cross-Validation Script Approach

Create `scripts/cross-validate.py` (Python) that:

1. Converts a `.cgt` file to each calculator's format
2. Runs each calculator
3. Compares results (total gains/losses per tax year)
4. Reports discrepancies > £1

All scripts are Python for consistency and easier date/JSON handling. Cross-validation is local/manual only (no CI integration).

**One-time audit**: Run on all 33 existing fixtures to validate manual calculations.

### Decision 4: Edge Cases to Add

Based on HMRC forums and guidance, prioritize these edge cases:

| Case                         | Description                      | Source                |
| ---------------------------- | -------------------------------- | --------------------- |
| Multi-currency same-day      | Buy in USD, sell in GBP same day | Common query          |
| Partial B&B with S104        | Sale matches both B&B and pool   | CG51560               |
| Zero-cost acquisition        | Bonus shares, spin-offs          | CG51746               |
| Negative pool (error case)   | Overselling validation           | User error            |
| Same-day buy/sell/buy        | Multiple trades same day         | Forum query           |
| 30-day boundary              | B&B on day 30 vs day 31          | CG51560               |
| Multiple tickers same day    | Ensures ticker isolation         | Implementation detail |
| Capital return > cost basis  | Creates gain                     | CG58620               |
| Split then immediate sell    | Tests split-then-match order     | CG51746               |
| Accumulation dividend timing | Affects cost basis date          | ERSM                  |

### Decision 5: Complex Multi-Year Fixture

Create `tests/inputs/RealisticMultiYear.cgt` with:

- 2-3 tax years of activity
- Multiple tickers (3-5)
- Mix of: buys, sells, dividends, capital returns, splits
- B&B matches
- Same-day matches
- S104 pool matches
- Multi-currency transactions
- Total ~50-100 transactions

This fixture will be verified against both external calculators.

## Risks / Trade-offs

### Risk: External Calculator Differences

External calculators may implement HMRC rules slightly differently (rounding, edge cases).

**Mitigation**: Document any known differences. Accept small rounding discrepancies (< £1). Investigate and document larger differences.

### Risk: Test Extraction May Break Private Access

Some inline tests may test private functions or `pub(crate)` items.

**Mitigation**: Integration tests (`tests/` directory) can't access private items. Keep unit tests for private functions inline if needed, or expose via `#[cfg(test)]` pub helpers.

## Migration Plan

1. **Phase 1**: Add coverage tooling (no code changes)
2. **Phase 2**: Move tests from smallest files first (config, exemption)
3. **Phase 3**: Move tests from larger files (validation, server)
4. **Phase 4**: Add edge case tests
5. **Phase 5**: Create cross-validation scripts
6. **Phase 6**: Create complex multi-year fixture

Each phase is independently mergeable.

## Resolved Questions

1. **Coverage reports**: Keep local only (no Codecov/CI integration)
2. **Cross-validation**: Keep local/manual only (no CI automation)
3. **Precision threshold**: £1 for cross-validation comparison
