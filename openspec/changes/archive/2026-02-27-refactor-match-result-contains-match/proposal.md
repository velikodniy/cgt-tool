## Why

`MatchResult` (internal matcher output) duplicates all five fields of `Match` (the serialized model) plus disposal metadata. The conversion in `group_matches_into_disposals` manually copies fields one by one, which is fragile and will silently drift if either type gains new fields.

## What Changes

- Restructure `MatchResult` to embed a `Match` directly instead of duplicating its fields
- Simplify `group_matches_into_disposals` to extract the inner `Match` instead of field-by-field mapping
- Update all `MatchResult` constructors in `same_day.rs`, `bed_and_breakfast.rs`, and `section104.rs`

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is a pure internal refactor — no spec-level behavior changes. The `Match` and `Disposal` serialization formats remain identical, and all existing golden-file tests must continue to pass unchanged.

## Impact

- `crates/cgt-core/src/matcher/mod.rs` — `MatchResult` struct definition
- `crates/cgt-core/src/matcher/same_day.rs` — constructor site
- `crates/cgt-core/src/matcher/bed_and_breakfast.rs` — constructor site
- `crates/cgt-core/src/matcher/section104.rs` — constructor site
- `crates/cgt-core/src/calculator.rs` — `group_matches_into_disposals` conversion and field access sites

## Verification

All existing golden-file tests (`cargo test`) and clippy lints must pass without modification. No behavioral change is expected.
