## Why

`TaxYearSummary::disposal_count` is a stored field that always equals `disposals.len() as u32`. Storing a derived value alongside its source creates a potential desync risk and violates the single source of truth principle. Replacing the field with a derived method eliminates this redundancy.

## What Changes

- Remove `disposal_count: u32` field from `TaxYearSummary` struct
- Add `disposal_count()` method that returns `self.disposals.len() as u32`
- Preserve JSON serialization output via custom `Serialize` impl to maintain backward compatibility with golden files and consumers
- Update all call sites that construct `TaxYearSummary` (remove field assignment)
- Update formatter call sites to use method syntax

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `cgt-calculation`: `TaxYearSummary` changes from a stored `disposal_count` field to a derived method, altering struct construction but not observable behavior

## Impact

- `crates/cgt-core/src/models.rs` -- struct definition changes
- `crates/cgt-core/src/calculator.rs` -- remove field assignment at construction sites
- `crates/cgt-formatter-plain/src/lib.rs` -- field access becomes method call
- `crates/cgt-formatter-pdf/src/lib.rs` -- field access becomes method call
- `crates/cgt-formatter-plain/tests/lib_tests.rs` -- remove field from test struct literals
- `crates/cgt-formatter-pdf/src/lib.rs` (tests) -- remove field from test struct literals
- No HMRC rule logic is affected; this is purely a structural refactoring
