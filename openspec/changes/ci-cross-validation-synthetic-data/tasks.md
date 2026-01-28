## 1. CI Workflow Setup

- [x] 1.1 Create `.github/workflows/cross-validate.yml` with workflow_dispatch trigger
- [x] 1.2 Add `cross-validate` job (macos-latest): checkout, setup Rust, build cgt-tool, setup Python/uv
- [x] 1.3 Build cgtcalc from source and add to PATH
- [x] 1.4 Add cross-validation run step calling `python3 scripts/cross-validate.py tests/inputs/*.cgt`
- [ ] 1.5 Test workflow manually via workflow_dispatch (requires GitHub push)

## 2. Synthetic CGT Fixture

- [x] 2.1 Create `tests/inputs/SyntheticComplex.cgt` header with test documentation
- [x] 2.2 Add ACME (USD) RSU vesting pattern: multi-award same-day vests, sell-to-cover sequences
- [x] 2.3 Add BETA (USD) trading pattern: Same Day, B&B, partial B&B, S104 pool scenarios
- [x] 2.4 Add GAMA (GBP) trading pattern: multi-currency transactions
- [x] 2.5 Add corporate actions: SPLIT for BETA, CAPRETURN for ACME
- [x] 2.6 Add tax year boundary transactions (April 5/6 edge cases)
- [x] 2.7 Add exact 30-day B&B boundary cases (D+30 matches, D+31 doesn't)
- [x] 2.8 Verify fixture passes `cargo run -- report tests/inputs/SyntheticComplex.cgt`

## 3. Schwab JSON Test Fixtures

- [x] 3.1 Create `tests/schwab/` directory
- [x] 3.2 Create `tests/schwab/synthetic-awards.json` with Schwab awards format structure
- [x] 3.3 Add Lapse events matching ACME RSU vest dates with FMV, SalePrice, tax withholding
- [x] 3.4 Add multi-award same-day vesting (4+ awards on one date)
- [x] 3.5 Create `tests/schwab/synthetic-transactions.json` with Schwab transactions format
- [x] 3.6 Add Stock Plan Activity entries for RSU settlements
- [x] 3.7 Add Sell entries for sell-to-cover with fees
- [x] 3.8 Add Journal entries for tax withholding
- [x] 3.9 Verify converter produces equivalent output: `cgt convert schwab tests/schwab/synthetic-transactions.json --awards tests/schwab/synthetic-awards.json`

## 4. Golden Files

- [x] 4.1 Generate `tests/json/SyntheticComplex.json` from fixture
- [x] 4.2 Generate `tests/plain/SyntheticComplex.txt` from fixture
- [x] 4.3 Verify golden files match by running `cargo test`

## 5. Cross-Validation Verification

- [x] 5.1 Run cross-validate.py locally against SyntheticComplex.cgt
- [x] 5.2 Document any expected discrepancies in fixture header comments
- [ ] 5.3 Verify CI workflow completes successfully with new fixture (requires GitHub push)
