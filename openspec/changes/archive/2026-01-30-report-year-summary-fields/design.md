## Context

The report summary rows in plain text and PDF currently show tax year, net gain, gross proceeds, exemption, and taxable gain. The JSON report mirrors `TaxReport` directly, which exposes total gain/loss and disposals but does not include an explicit disposal count. Accountants need per-year disposal count and separate gains-before-losses and losses totals surfaced consistently across all formats. CG51560 treats same-day disposals as a single transaction, so the disposal count should follow the grouped disposal definition.

The core tax logic already computes per-disposal net gain/loss and aggregates `total_gain`, `total_loss`, and `net_gain` per tax year. The change is about exposing these aggregates and disposal counts in summaries, without altering tax calculations.

## Goals / Non-Goals

**Goals:**

- Add a per-year disposal count to the core report output so JSON includes it directly.
- Surface disposal count, net gain, total gains (before losses), and total losses in plain and PDF summary tables.
- Keep all formats aligned on the same per-year values.

**Non-Goals:**

- Changing any tax calculation logic, matching rules, or gain/loss computation.
- Introducing new report sections beyond the summary row changes.
- Altering currency formatting behavior outside the new summary columns.

## Decisions

1. **Add `disposal_count` to `TaxYearSummary` in cgt-core, defined as grouped disposals per tax year.**

   - **Why:** JSON output is a direct serialization of `TaxReport`; adding a field ensures JSON, plain, and PDF can share a single source of truth. Same-day disposals are treated as a single transaction per CG51560, so the grouped count aligns with matching rules and accountant expectations.
   - **Alternatives:** Count raw SELL transactions. Rejected because same-day disposals would be double-counted versus CG51560 grouping.

2. **Keep existing `net_gain` in summary output and add `total_gain` and `total_loss`.**

   - **Why:** `net_gain` remains the headline figure users expect; totals provide the requested breakdown. Reuse avoids duplication and ensures parity with JSON totals.
   - **Alternatives:** Recompute gains/losses within formatters. Rejected to avoid inconsistent rounding and logic divergence.

3. **Keep summary column order consistent across plain and PDF.**

   - **Why:** Visual parity makes reconciliation easier and avoids confusion between formats.
   - **Alternatives:** Format-specific ordering. Rejected for consistency requirements.

## Risks / Trade-offs

- **[Golden file churn]** → Update JSON and plain/PDF fixtures in one pass; validate diffs against known totals.
- **[Column width constraints in PDF]** → Adjust PDF table column widths to fit new columns without truncation.
- **[Count semantics]** → Disposal count follows same-day aggregation; document explicitly in specs and tests.

## Migration Plan

- No runtime migration. Change is additive to serialized output; update fixtures and release notes as part of the change.

## Open Questions

- None.
