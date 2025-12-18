# Tasks

## 1. Documentation Updates

- [x] 1.1 Update TAX_RULES.md to clarify gross vs net proceeds for SA108 compliance
- [x] 1.2 Add section explaining which value goes in SA108 Box 21 vs Box 22

## 2. Data Model Updates

- [x] 2.1 Add `gross_proceeds` field to `Disposal` struct in `cgt-core/src/models.rs`
- [x] 2.2 Update matcher modules to populate both gross and net proceeds

## 3. Plain Text Formatter Updates

- [x] 3.1 Update summary table to show gross proceeds (for SA108 Box 21 compatibility)
- [x] 3.2 Update disposal details to show both gross and net proceeds with labels
- [x] 3.3 Fix same-day merge display to show weighted average price
- [x] 3.4 Add footnote/legend explaining gross vs net

## 4. PDF Formatter Updates

- [x] 4.1 Update summary table to match plain text (show gross proceeds)
- [x] 4.2 Update disposal details to show both gross and net proceeds
- [x] 4.3 Fix same-day merge display to show weighted average price
- [x] 4.4 Update Typst template with new fields

## 5. Test Updates

- [x] 5.1 Update all `tests/plain/*.txt` expected outputs with new format
- [x] 5.2 Update all `tests/json/*.json` expected outputs with new field
- [x] 5.3 Verify all existing tests pass with new format

## 6. Validation

- [x] 6.1 Run `cargo test` and ensure all tests pass
- [x] 6.2 Run `cargo clippy` with no warnings
- [x] 6.3 Manually verify output matches reference files for key test cases
