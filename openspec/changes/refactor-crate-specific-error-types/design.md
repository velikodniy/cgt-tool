## Context

`cgt-core::CgtError` currently contains three non-core variants: `InvalidCurrencyCode`, `PdfGeneration`, and `IoError`.
Current usage review shows:

- `PdfGeneration` is constructed only in `crates/cgt-formatter-pdf/src/lib.rs`.
- `InvalidCurrencyCode` is not constructed anywhere.
- `IoError` is not constructed from `cgt-core` runtime paths.

This indicates that `CgtError` has become broader than `cgt-core` ownership boundaries.

## Goals / Non-Goals

**Goals:**

- Keep `CgtError` scoped to parse/calculation/config errors owned by `cgt-core`.
- Move PDF-specific failures to `cgt-formatter-pdf`.
- Preserve clear user-facing failure messages and avoid behavior regressions.
- Keep the refactor minimal and low-risk.

**Non-Goals:**

- Redesigning the global error architecture across all crates.
- Changing tax calculation logic or matching behavior.
- Introducing broad trait/API redesigns where a local type change is sufficient.

## Decisions

1. Remove `InvalidCurrencyCode`, `PdfGeneration`, and `IoError` from `cgt-core::CgtError`.

   - Rationale: these variants are not core-owned concerns.
   - Alternative considered: keep variants and document ownership; rejected because ownership remains ambiguous.

2. Introduce a `PdfError` enum in `cgt-formatter-pdf` and return `Result<Vec<u8>, PdfError>` from `format`.

   - Rationale: PDF generation failures should be represented by formatter-owned errors.
   - Alternative considered: keep returning `CgtError` and map internally; rejected because it preserves cross-crate coupling.

3. Keep call-site impact minimal by relying on `anyhow` conversion in CLI (`?` on `PdfError`).

   - Rationale: no user-facing command flow changes needed.
   - Alternative considered: introduce wrapper error type in CLI; rejected as unnecessary for this scope.

## Risks / Trade-offs

- \[Public API change for `cgt-formatter-pdf::format`\] -> Mitigation: keep error strings explicit and preserve failure semantics so CLI behavior stays stable.
- \[Unused assumptions about removed `CgtError` variants elsewhere\] -> Mitigation: workspace `cargo clippy` and `cargo test` to catch compile/runtime regressions.

## Migration Plan

1. Update `cgt-core::CgtError` to remove non-core variants.
2. Add `PdfError` in `cgt-formatter-pdf` and migrate internal helpers to use it.
3. Update docs/comments and any imports affected by signature changes.
4. Run `cargo fmt`, `cargo clippy`, and `cargo test`.

## Open Questions

- None for this change; implementation is local and bounded.
