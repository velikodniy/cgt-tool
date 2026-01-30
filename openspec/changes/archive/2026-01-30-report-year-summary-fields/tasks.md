## 1. Core report data

- [x] 1.1 Add `disposal_count` to `TaxYearSummary` and populate it in `cgt-core` calculation output
- [x] 1.2 Update JSON golden files in `tests/json` to include `disposal_count` for affected fixtures
- [x] 1.3 Run `cargo fmt && cargo clippy && cargo test`

## 2. Plain formatter summary

- [x] 2.1 Extend plain summary header/rows to include disposal count, total gains, and total losses per tax year
- [x] 2.2 Update `tests/plain` golden outputs and any formatter unit tests for the new columns
- [x] 2.3 Run `cargo fmt && cargo clippy && cargo test`

## 3. PDF formatter summary

- [x] 3.1 Add disposal count, total gains, and total losses to PDF summary data and Typst table columns
- [x] 3.2 Update PDF formatter tests to cover the new summary fields
- [x] 3.3 Run `cargo fmt && cargo clippy && cargo test`

## 4. Consistency verification

- [x] 4.1 Generate a multi-year report in plain/JSON/PDF and verify summary values match across formats
