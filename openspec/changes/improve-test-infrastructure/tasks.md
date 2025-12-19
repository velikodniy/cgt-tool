# Tasks: Improve Test Infrastructure

## 1. Coverage Tooling

- [ ] 1.1 Install and configure cargo-llvm-cov
- [ ] 1.2 Run initial coverage report to establish baseline
- [ ] 1.3 Document coverage commands in AGENTS.md

## 2. Move Inline Tests - cgt-core

- [ ] 2.1 Create `crates/cgt-core/tests/lib_tests.rs` (from lib.rs, 4 tests)
- [ ] 2.2 Create `crates/cgt-core/tests/config_tests.rs` (from config.rs, 5 tests)
- [ ] 2.3 Create `crates/cgt-core/tests/exemption_tests.rs` (from exemption.rs, 3 tests)
- [ ] 2.4 Create `crates/cgt-core/tests/validation_tests.rs` (from validation.rs, 9 tests)
- [ ] 2.5 Remove inline test modules from cgt-core source files
- [ ] 2.6 Run `cargo test` to verify all tests pass

## 3. Move Inline Tests - cgt-money

- [ ] 3.1 Verify existing tests in `tests/` directory are complete
- [ ] 3.2 Check for any inline tests to move

## 4. Move Inline Tests - cgt-format

- [ ] 4.1 Create `crates/cgt-format/tests/lib_tests.rs` (from lib.rs, 11 tests)
- [ ] 4.2 Remove inline test module from lib.rs
- [ ] 4.3 Run `cargo test -p cgt-format` to verify

## 5. Move Inline Tests - cgt-formatter-plain

- [ ] 5.1 Create `crates/cgt-formatter-plain/tests/lib_tests.rs` (from lib.rs, 4 tests)
- [ ] 5.2 Remove inline test module from lib.rs
- [ ] 5.3 Run `cargo test -p cgt-formatter-plain` to verify

## 6. Move Inline Tests - cgt-formatter-pdf

- [ ] 6.1 Create `crates/cgt-formatter-pdf/tests/lib_tests.rs` (from lib.rs, 3 tests)
- [ ] 6.2 Remove inline test module from lib.rs
- [ ] 6.3 Run `cargo test -p cgt-formatter-pdf` to verify

## 7. Move Inline Tests - cgt-converter

- [ ] 7.1 Create `crates/cgt-converter/tests/output_tests.rs` (from output.rs, 5 tests)
- [ ] 7.2 Create `crates/cgt-converter/tests/schwab_tests.rs` (from schwab/mod.rs, 22 tests)
- [ ] 7.3 Create `crates/cgt-converter/tests/awards_tests.rs` (from schwab/awards.rs, 26 tests)
- [ ] 7.4 Remove inline test modules from source files
- [ ] 7.5 Run `cargo test -p cgt-converter` to verify

## 8. Move Inline Tests - cgt-mcp

- [ ] 8.1 Create `crates/cgt-mcp/tests/resources_tests.rs` (from resources.rs, 5 tests)
- [ ] 8.2 Create `crates/cgt-mcp/tests/server_tests.rs` (from server.rs, 39 tests)
- [ ] 8.3 Remove inline test modules from source files
- [ ] 8.4 Run `cargo test -p cgt-mcp` to verify

## 9. Edge Case Tests

- [ ] 9.1 Create `tests/inputs/MultiCurrencySameDay.cgt` with multi-currency same-day scenario
- [ ] 9.2 Create `tests/inputs/PartialBnBWithS104.cgt` for partial B&B with S104 fallback
- [ ] 9.3 Create `tests/inputs/BnBBoundary30Days.cgt` for day 30 vs day 31 boundary
- [ ] 9.4 Create `tests/inputs/SameDayBuySellBuy.cgt` for complex same-day scenarios
- [ ] 9.5 Create `tests/inputs/CapReturnExceedsCost.cgt` for capital return exceeding cost
- [ ] 9.6 Create `tests/inputs/SplitThenSell.cgt` for split immediately followed by sell
- [ ] 9.7 Add corresponding expected JSON outputs for each fixture
- [ ] 9.8 Add corresponding expected plain text outputs for each fixture

## 10. Cross-Validation Scripts

- [ ] 10.1 Create `scripts/convert-to-raw.py` to convert our .cgt files to KapJI RAW format
- [ ] 10.2 Create `scripts/convert-to-cgtcalc.py` to convert our .cgt files to cgtcalc format
- [ ] 10.3 Create `scripts/cross-validate.py` main driver script (Python)
- [ ] 10.4 Document cross-validation process in scripts/README.md

## 10a. Audit Existing Fixtures (One-Time)

- [ ] 10a.1 Run cross-validation on all 33 existing .cgt fixtures
- [ ] 10a.2 Compare results against existing JSON/plain outputs
- [ ] 10a.3 Document any discrepancies (> £1 threshold)
- [ ] 10a.4 For discrepancies: determine which calculator is correct
- [ ] 10a.5 Update fixture expected outputs if cgt-tool was wrong
- [ ] 10a.6 Document external calculator bugs/differences if they were wrong

## 11. Complex Multi-Year Fixture

- [ ] 11.1 Design realistic multi-year scenario (document in test file comments)
- [ ] 11.2 Create `tests/inputs/RealisticMultiYear.cgt` with 50-100 transactions
- [ ] 11.3 Calculate expected results manually or via cross-validation
- [ ] 11.4 Create `tests/json/RealisticMultiYear.json` expected output
- [ ] 11.5 Create `tests/plain/RealisticMultiYear.txt` expected output

## 12. Documentation

- [ ] 12.1 Document coverage commands in AGENTS.md
- [ ] 12.2 Document cross-validation usage in scripts/README.md

## 13. Final Verification

- [ ] 13.1 Run full test suite: `cargo test`
- [ ] 13.2 Run clippy: `cargo clippy`
- [ ] 13.3 Run coverage and verify improvement from baseline
- [ ] 13.4 Update AGENTS.md with new test commands if needed
