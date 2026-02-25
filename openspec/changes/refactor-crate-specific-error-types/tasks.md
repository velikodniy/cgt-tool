## 1. Error boundary refactor

- [x] 1.1 Remove non-core variants (`InvalidCurrencyCode`, `PdfGeneration`, `IoError`) from `cgt-core::CgtError` and update affected imports/usages.
- [x] 1.2 Introduce `PdfError` in `cgt-formatter-pdf` and migrate helper/entry-point return types to formatter-owned errors.

## 2. Integration and verification

- [x] 2.1 Update CLI call paths and docs/comments impacted by PDF formatter signature changes while preserving user-facing behavior.
- [x] 2.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`; fix any regressions.
