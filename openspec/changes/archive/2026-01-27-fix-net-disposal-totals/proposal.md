## Why

Tax year totals currently sum positive and negative match legs separately, which overstates both gains and losses when a single disposal has mixed match outcomes (e.g., Same Day loss plus B&B gain). HMRC treats each disposal as a single net result after applying CG51560 matching rules and allowable expenditure per TCGA92/S38 (CG15150/CG15250), so the tool’s totals should reflect net per-disposal outcomes.

## What Changes

- Compute `total_gain` and `total_loss` per tax year from net disposal results (proceeds minus allowable costs after matching), not from individual match legs; net gain remains the sum of disposal nets.
- Add a note in the plain report summary clarifying that gains/losses are net per disposal for HMRC/SA108 reporting.
- Update documentation to clarify that reporting totals follows net-per-disposal treatment per HMRC/SA108 expectations.
- Add tests for mixed-rule disposals (Same Day + B&B / Same Day + S104) to ensure totals are derived from net disposal results.

## Capabilities

### New Capabilities

- None

### Modified Capabilities

- `cgt-calculation`: Define tax-year total gain/loss as the sum of net results per disposal (gain if net ≥ 0, loss if net < 0), aligned with CG51560 matching and TCGA92/S38 allowable expenditure.
- `plain-formatter`: Clarify in the summary note that gains/losses reflect net per-disposal results.

## Impact

- `cgt-core` tax year aggregation (`calculate_totals`) and any report consumers that display `total_gain` / `total_loss` (CLI, formatters, WASM JSON).
- `cgt-formatter-plain` summary note and its spec requirements.
- Documentation updates in `docs/tax-rules.md` for SA108-aligned reporting.
- New regression tests in `crates/cgt-core` for mixed-match disposals.

## Verification

- Add unit tests that construct disposals with mixed match legs and assert totals use net per-disposal results.
- Validate against HMRC guidance references (CG51560, CG15150, CG15250) and SA108 reporting rules in `docs/tax-rules.md`.
