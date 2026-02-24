## Why

`ValidationError` and `ValidationWarning` in `cgt-core` are structurally identical types (same fields: `line`, `date`, `ticker`, `message`) with nearly identical `Display` impls differing only in the "Error"/"Warning" prefix. This is ~60 lines of unnecessary duplication that increases maintenance burden and makes future field additions require changes in two places.

## What Changes

- Replace `ValidationError` and `ValidationWarning` with a single `ValidationIssue` struct containing a `Severity` enum field
- Unify the two `Display` impls into one that uses the severity for the prefix
- Update `ValidationResult` fields and all consumers (cgt-core re-exports, cgt-wasm JSON bridge)
- **BREAKING**: Public API types `ValidationError` and `ValidationWarning` are removed in favor of `Severity` and `ValidationIssue`

## Capabilities

### New Capabilities

(none — this is a pure refactor of existing internal types)

### Modified Capabilities

(none — no spec-level behavior changes; validation still produces errors and warnings with the same semantics)

## Impact

- `crates/cgt-core/src/validation.rs` — primary change location
- `crates/cgt-core/src/lib.rs` — re-export update
- `crates/cgt-wasm/src/lib.rs` — JSON bridge `From` impls
- `crates/cgt-core/tests/validation_tests.rs` — tests access `.errors` and `.warnings` fields on `ValidationResult`

## Verification

All existing tests must pass unchanged. `cargo fmt && cargo clippy && cargo test` must succeed. The refactored types produce identical `Display` output and identical validation behavior.
