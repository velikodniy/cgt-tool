## Why

`TaxPeriod::from_date()` constructs `Self(year)` directly, bypassing the `[1900, 2100]` range validation that `new()` enforces. This means a date before April 6, 1900 produces `TaxPeriod(1899)` — a value that `new()` would reject. Two constructors for the same type disagree on what values are valid, violating the type's invariant.

## What Changes

- Make `TaxPeriod::from_date()` return `Result<Self, CgtError>` instead of `Self`, calling `new()` internally to enforce the same validation.
- Update all callers of `from_date()` in the calculator to propagate the error.
- Add a test verifying that out-of-range dates produce an error.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `cgt-calculation`: `TaxPeriod::from_date()` becomes fallible, returning `Result<Self, CgtError>`.

## Impact

- `crates/cgt-core/src/models.rs`: Signature change on `from_date()`.
- `crates/cgt-core/src/calculator.rs`: Two call sites must propagate the `Result` with `?`.
- `crates/cgt-core/tests/lib_tests.rs`: Existing tests must call `.unwrap()` (test code only) on the new `Result`; new test added for out-of-range dates.

## Verification

The fix is validated by:

1. Existing `test_tax_period_from_date` tests continue to pass (valid dates still work).
2. A new test confirms that a date before 1900-04-06 returns `CgtError::InvalidTaxYear`.
3. `cargo clippy` confirms no `unwrap` in production code.
