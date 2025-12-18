# Change: Fix report proceeds display and same-day merge bug

## Why

Comparison of trusted reference test outputs against our plain text outputs revealed three issues:

1. **Proceeds shown as net instead of gross**: Reports display net proceeds (after fees) but SA108 Box 21 "Disposal proceeds" requires gross sale value. The reference files correctly show gross proceeds.

2. **Same-day merge display bug**: When multiple same-day sales are merged, the displayed per-share price shows one of the individual prices instead of the weighted average.

3. **Loss carry-forward not applied** (out of scope): The `CarryLoss.txt` test shows the taxable gain calculation doesn't apply carried losses. This is a separate calculation issue that should be addressed in a different change.

## What Changes

- **MODIFIED**: Plain text formatter to display both gross and net proceeds with clear labels
- **MODIFIED**: PDF formatter to match plain text formatter proceeds display
- **MODIFIED**: Same-day match proceeds calculation to use weighted average price when multiple sells occur
- **ADDED**: Additional test cases for multi-sell same-day scenarios
- **MODIFIED**: TAX_RULES.md to clarify gross vs net proceeds for SA108 compliance

## Impact

- Affected specs: `plain-formatter`, `pdf-formatter`
- Affected code:
  - `crates/cgt-formatter-plain/src/lib.rs`
  - `crates/cgt-formatter-pdf/src/lib.rs`
  - `crates/cgt-formatter-pdf/src/templates/report.typ`
  - `TAX_RULES.md`
  - `tests/plain/*.txt` (expected outputs need updating)
