## Context

Current tax-year totals are derived by summing positive and negative match legs across all disposals. When a single disposal is matched across multiple rules (CG51560) and those legs produce both gains and losses, the totals overstate both sides even though the disposal itself has a single net result. HMRC computation treats a disposal as proceeds less allowable expenditure (TCGA92/S38; CG15150, CG15250), so reporting should classify each disposal as either a gain or a loss after matching has been applied.

This change is cross-cutting: the core aggregation logic lives in `cgt-core`, but totals are surfaced through CLI, formatters, and WASM JSON reports. Matching logic (Same Day, B&B, Section 104) is correct and remains unchanged.

## Goals / Non-Goals

**Goals:**

- Compute `total_gain` and `total_loss` from net disposal results per tax year, where each disposal contributes either a gain or a loss after applying CG51560 matching and TCGA92/S38 allowable expenditure.
- Preserve per-match details for auditability while ensuring summary totals reflect HMRC reporting expectations (SA108-style net per disposal).
- Add targeted tests for mixed-rule disposals to prevent regressions.
- Add a summary note in plain reports clarifying that gains/losses are net per disposal.

**Non-Goals:**

- Do not change matching order or logic (Same Day, B&B, Section 104).
- Do not change disposal grouping, proceeds/allowable cost calculations, or per-match gain/loss values.
- Do not introduce new report fields or alter output formats beyond the corrected totals and a clarifying summary note.

## Decisions

1. **Aggregate totals per disposal, not per match leg.**

   - **Decision:** Update `calculate_totals` to iterate over disposals and sum the net of all matches within each disposal. If net >= 0, add to `total_gain`; if net < 0, add abs(net) to `total_loss`.
   - **Rationale:** HMRC computes gain or loss for a disposal as proceeds minus allowable costs (TCGA92/S38; CG15150, CG15250). A disposal can span multiple match rules (CG51560) but still yields a single net result.
   - **Alternative considered:** Keep current per-leg totals and add parallel fields (e.g., `total_gain_legs`, `total_loss_legs`) for diagnostics. Rejected for now to avoid schema and formatter changes.

2. **Keep match-level detail intact for audit trails.**

   - **Decision:** Do not alter `Match` or `Disposal` structures; only adjust how totals are derived from existing data.
   - **Rationale:** Existing outputs rely on match breakdowns; retaining them preserves transparency while fixing summary totals.

3. **Tests focus on mixed-rule disposals.**

   - **Decision:** Add unit tests that construct disposals with mixed positive/negative match legs (e.g., Same Day loss + B&B gain; Same Day loss + S104 gain) and assert that totals are based on net disposal results.
   - **Rationale:** The bug manifests only when a single disposal has both gain and loss legs; targeted tests directly cover this corner case.

4. **Keep per-leg totals internal; add a plain summary note instead.**

   - **Decision:** Do not add per-leg gain/loss totals to reports for this change. Keep match-level details for auditability, and add a short note in the plain summary clarifying that totals are net per disposal.
   - **Rationale:** The fix is about HMRC-aligned reporting; match details already expose per-leg results when needed. A summary note reduces confusion without expanding report schemas.

## Risks / Trade-offs

- **[Risk]** Existing golden tests or downstream tooling may depend on the inflated totals. → **Mitigation:** Update expectations and document the corrected HMRC-aligned definition in `docs/tax-rules.md`.
- **[Risk]** Net-zero disposals could oscillate due to rounding. → **Mitigation:** Use the same `Decimal` precision already used for match values and treat exact zero as neither gain nor loss.
- **[Trade-off]** Removing per-leg totals loses a quick view of gross gains vs gross losses from mixed disposals. → **Mitigation:** Match-level data remains visible in detailed disposal output.

## Migration Plan

1. Update `cgt-core` aggregation (`calculate_totals`) to compute per-disposal net totals.
2. Add unit tests for mixed-match disposals in `crates/cgt-core`.
3. Update `docs/tax-rules.md` to clarify totals are net per disposal and align with SA108 reporting expectations.
4. Update the plain report summary note (and its spec) to label gains/losses as net per disposal.
5. Run tests and review any output changes; update golden files only after manual validation confirms the totals align with HMRC net-per-disposal reporting.

## Open Questions

- None at this time.
