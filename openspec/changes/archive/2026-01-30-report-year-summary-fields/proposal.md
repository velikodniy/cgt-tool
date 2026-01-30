## Why

Accountants need per-tax-year totals for disposals, gains before losses, and losses in the summary. Today those figures are only visible by scanning detailed sections or derived externally, which makes reconciliation slower and error-prone.

## What Changes

- Add per-tax-year disposal counts to the calculated report data so JSON output can include them alongside existing totals.
- Expand plain text summary rows to include disposal count, total gains (before losses), and total losses per year.
- Expand PDF summary table with the same per-year fields as plain text and JSON.
- Ensure all report formats expose the same per-year summary data.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `cgt-calculation`: extend per-year summary output to include disposal count alongside existing total gain/loss fields.
- `plain-formatter`: summary table includes disposal count, total gains, and total losses per tax year.
- `pdf-formatter`: summary table includes disposal count, total gains, and total losses per tax year.

## Impact

- Update `TaxYearSummary` data output and JSON fixtures.
- Update plain and PDF summary layout and tests for new columns.
- Keep core tax calculation logic unchanged; only summary exposure and formatting change.

## Verification

- Update and run data-driven JSON golden tests to include disposal count and validate totals per year.
- Update plain/PDF formatter tests and golden outputs to match the new summary columns.
- Cross-check all three formats for a sample multi-year file to ensure identical per-year values.
