# Change: Fix Proceeds Display

## Why

- Plain report proceeds lines show expressions like `10 × £4.6702 = £34` while the net proceeds already deduct sale fees (£12.50), so the displayed math is wrong and suggests a miscalculation.
- The mismatch between the per-share price expression and the net proceeds (and JSON output) erodes trust in the calculation and conflicts with HMRC guidance that sale expenses reduce proceeds.
- Flooring currency values worsens the discrepancy (e.g., £34.202 shown as £34), making it harder to trace losses such as the £19.863 loss in MultipleMatches.

## What Changes

- Update disposal calculation lines to show net proceeds as `quantity × unit price - sale expenses = net proceeds`, matching the computed proceeds value and omitting the `- £0` term when expenses are zero.
- Align proceeds display with the currency policy for each currency’s recommended precision (minor units) so the shown net proceeds equals the underlying calculation, and apply this consistently across outputs.
- Refresh formatter fixtures/tests (plain and PDF) and extend coverage to verify proceeds lines with sale expenses.

## Impact

- Specs: `plain-formatter` and `pdf-formatter` (add/clarify disposal calculation formatting requirement; reinforce currency policy for proceeds lines).
- Code (expected): `crates/cgt-formatter-plain` and `crates/cgt-formatter-pdf` to keep outputs aligned.
- Tests/fixtures: `tests/plain/*` goldens and CLI golden comparison in `crates/cgt-cli/tests/cli_tests.rs`.

## Decisions

- Apply currency precision using each currency’s recommended minor units, across all outputs (not just proceeds lines).
- When sale expenses are zero, show `qty × price = net` (omit `- £0`).
