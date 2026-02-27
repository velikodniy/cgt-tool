## Why

Both plain and PDF formatters independently recompute the same derived CGT values from model data, which duplicates logic and increases the risk of formatter drift. Centralizing these calculations in `cgt-core` model methods reduces maintenance cost and keeps presentation layers focused on rendering.

## What Changes

- Add computed-value methods on core model types for disposal and tax-year summary aggregates used by formatters.
- Replace duplicate inline calculations in `cgt-formatter-plain` and `cgt-formatter-pdf` with calls to those model methods.
- Keep behavior unchanged by preserving existing formulas and output formatting.

## Capabilities

### New Capabilities

- `model-computed-values`: Provide canonical computed methods for commonly used disposal and tax-year aggregates.

### Modified Capabilities

- `plain-formatter`: Update formatter implementation to consume model computed methods instead of local aggregation logic.
- `pdf-formatter`: Update formatter implementation to consume model computed methods instead of local aggregation logic.

## Impact

- Affected code: `crates/cgt-core/src/model.rs`, `crates/cgt-formatter-plain/src/lib.rs`, `crates/cgt-formatter-pdf/src/lib.rs`.
- No DSL, CLI surface, or tax-rule behavior changes.
- Reduced duplication across formatter crates and clearer separation of responsibilities.

## Verification

- Run `cargo fmt`, `cargo clippy`, and `cargo test` to ensure lint-clean, behavior-preserving refactor.
- Validate formatter golden outputs remain unchanged through existing test suites.
