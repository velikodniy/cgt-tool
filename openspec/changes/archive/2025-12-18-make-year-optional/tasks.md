## 1. Core Calculation

- [x] 1.1 Modify `calculate()` function signature to accept `Option<i32>` for `tax_year_start`
- [x] 1.2 When `tax_year_start` is `None`, group all disposals by tax year instead of filtering
- [x] 1.3 Return `TaxReport` with multiple `TaxYearSummary` entries sorted by period

## 2. CLI Changes

- [x] 2.1 Change `--year` argument from required to optional in `commands.rs`
- [x] 2.2 Update `main.rs` to pass `Option<i32>` to calculator
- [x] 2.3 Ensure help text explains behavior when `--year` is omitted

## 3. MCP Server Changes

- [x] 3.1 Update `calculate_report` tool schema to make `year` optional
- [x] 3.2 Update tool description to explain all-years behavior
- [x] 3.3 Update handler to pass `Option<i32>` to calculator

## 4. Testing

- [x] 4.1 Add test case for all-years report generation
- [x] 4.2 Add test case verifying existing single-year behavior unchanged
- [x] 4.3 Verify plain text formatter handles multi-year reports correctly (already supported)
- [x] 4.4 Verify JSON output includes all years when `--year` omitted

## 5. Documentation

- [x] 5.1 Update README with new optional `--year` behavior
