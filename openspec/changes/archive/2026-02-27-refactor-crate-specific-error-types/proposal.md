## Why

`CgtError` in `cgt-core` currently includes variants that belong to other crates (`InvalidCurrencyCode`, `PdfGeneration`, `IoError`). This creates a cross-crate "god error" type that blurs ownership boundaries and makes core error semantics less clear.

## What Changes

- Remove non-core variants from `cgt-core`'s `CgtError` so it only represents errors created and owned by core parsing/calculation logic.
- Introduce or use crate-specific error types in crates that actually produce these errors (notably currency parsing and PDF formatting paths).
- Update call sites and conversion boundaries so user-visible behavior and messages remain stable while type ownership becomes explicit.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `pdf-formatter`: tighten error ownership so PDF-specific failures are reported by formatter-specific error types.

## Impact

- `crates/cgt-core/src/error.rs` and dependent signatures/imports that currently rely on non-core variants.
- Error definitions and propagation in crates that currently map to `CgtError::InvalidCurrencyCode`, `CgtError::PdfGeneration`, or `CgtError::IoError`.
- No tax-rule or matching-rule behavior changes.

## Verification

- Run `cargo fmt`, `cargo clippy`, and `cargo test` across the workspace.
- Validate that CLI/formatter error text remains clear and actionable.
