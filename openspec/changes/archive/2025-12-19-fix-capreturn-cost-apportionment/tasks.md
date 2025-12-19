# Tasks

## 1. Core Fix

- [x] 1.1 Fix `apply_cost_adjustment` in `acquisition_ledger.rs` to use `total_held` as denominator
- [x] 1.2 Regenerate all test outputs (`tests/plain/*.txt` and `tests/json/*.json`)
- [x] 1.3 Verify all existing tests pass

## 2. Documentation Updates

- [x] 2.1 Update TAX_RULES.md Capital Returns section with apportionment details
- [x] 2.2 Update cgt-calculation spec with clearer apportionment requirement

## 3. Additional Tests

- [x] 3.1 Add unit test for CAPRETURN with partial holdings (event_amount < total_held)
- [x] 3.2 Add unit test for CAPRETURN with multiple lots
- [x] 3.3 Add unit test for DIVIDEND with multiple lots
- [x] 3.4 Add unit test for CAPRETURN/DIVIDEND combined
- [x] 3.5 Add unit test for CAPRETURN not affecting later acquisitions
- [x] 3.6 Add unit test for CAPRETURN with prior partial sale (B&B interaction)

## 4. Validation

- [x] 4.1 Run `cargo test` - all tests pass
- [x] 4.2 Run `cargo clippy` - no warnings
- [x] 4.3 Verify discrepant test cases match reference values
- [x] 4.4 Run `openspec validate fix-capreturn-cost-apportionment --strict`
