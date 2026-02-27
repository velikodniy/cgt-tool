## Why

Date+ticker sorting logic is duplicated in `cgt-core` and both formatter crates. The current duplication increases maintenance cost and risks inconsistent ordering behavior when one copy changes but others do not.

## What Changes

- Add a small shared sorting utility for date+ticker ordering in `cgt-core`.
- Replace duplicated inline sorting closures in calculator and formatter call sites with the shared helper.
- Keep observable ordering behavior unchanged while reducing duplicate code paths.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `cgt-calculation`: clarify that gain/loss records and downstream formatter inputs use one canonical date+ticker ordering implementation.
- `plain-formatter`: clarify that transaction/result ordering reuses the canonical core ordering helper.
- `pdf-formatter`: clarify that transaction/result ordering reuses the canonical core ordering helper.

## Impact

- Affected code: `crates/cgt-core/src/calculator.rs`, `crates/cgt-formatter-plain/src/lib.rs`, `crates/cgt-formatter-pdf/src/lib.rs`, and new shared utility location in `cgt-core`.
- APIs: internal crate APIs only; no DSL, CLI, or output format schema changes.
- Dependencies: no new external dependencies.

## Verification

- Run `cargo fmt`, `cargo clippy`, and `cargo test` to confirm no regressions.
- Compare plain/PDF formatter golden tests to verify ordering output remains stable.
