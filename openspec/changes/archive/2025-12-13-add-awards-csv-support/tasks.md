# Tasks: Add CSV Format Support for Schwab Equity Awards

## 1. Awards Parser Updates

- [ ] 1.1 Add `parse_awards_csv` function for CSV format
- [ ] 1.2 Implement paired-row CSV parsing logic
- [ ] 1.3 Extract Symbol, AwardDate, FairMarketValuePrice from CSV
- [ ] 1.4 Add format detection based on filename extension
- [ ] 1.5 Rename `parse_awards_json` internal usage to `parse_awards` wrapper
- [ ] 1.6 Update error handling for CSV-specific errors

## 2. CSV Parser Implementation

- [ ] 2.1 Parse CSV header and validate required columns
- [ ] 2.2 Implement state machine for paired-row parsing
- [ ] 2.3 Handle transaction row (extract Symbol)
- [ ] 2.4 Handle award row (extract AwardDate, FairMarketValuePrice)
- [ ] 2.5 Parse dates in MM/DD/YYYY format
- [ ] 2.6 Parse FMV prices (handle $ prefix, commas)

## 3. Integration

- [ ] 3.1 Update `schwab/mod.rs` to pass filename to parser
- [ ] 3.2 Update CLI to extract filename from awards path
- [ ] 3.3 Pass filename through SchwabInput or parse call

## 4. Tests

- [ ] 4.1 Create CSV awards test fixture (synthetic data)
- [ ] 4.2 Add test for basic CSV parsing
- [ ] 4.3 Add test for paired-row parsing
- [ ] 4.4 Add test for format detection (JSON vs CSV)
- [ ] 4.5 Add test for CSV with multiple awards
- [ ] 4.6 Add test for missing required columns in CSV
- [ ] 4.7 Add test for invalid CSV date format
- [ ] 4.8 Add test for invalid CSV price format
- [ ] 4.9 Add integration test: transactions + CSV awards

## 5. Documentation

- [ ] 5.1 Update README.md to mention CSV support
- [ ] 5.2 Add example of CSV awards usage
- [ ] 5.3 Update awards file documentation

## 6. Validation

- [ ] 6.1 Run `cargo clippy` — fix all warnings
- [ ] 6.2 Run `cargo fmt` — format code
- [ ] 6.3 Run full test suite
- [ ] 6.4 Manual test with synthetic CSV awards file
