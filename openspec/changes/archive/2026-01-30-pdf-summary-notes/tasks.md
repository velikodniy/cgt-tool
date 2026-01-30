## 1. Typst Template Updates

- [x] 1.1 Update `crates/cgt-formatter-pdf/src/templates/report.typ` summary table headers to use `#super("1")`, `#super("2")`, `#super("3")` for footnote markers
- [x] 1.2 Replace the list-based notes in `report.typ` with a numbered list block corresponding to the footnotes (1. grouped disposals, 2. SA108 Proceeds, 3. SA108 Gains/Losses)
- [x] 1.3 Run `cargo run -- report --format pdf ...` to generate a sample report and verify layout

## 2. Verification

- [x] 2.1 Regenerate a sample report (e.g. `tests/inputs/RealisticMultiYear.cgt`) and check visual alignment
- [x] 2.2 Run `cargo fmt && cargo clippy && cargo test` to ensure no regressions
