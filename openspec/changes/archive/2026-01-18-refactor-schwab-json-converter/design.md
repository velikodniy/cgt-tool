## Context

Schwab now exports transactions and equity awards as JSON with a different schema than the current CSV/JSON structures supported by the converter. We need a JSON-only pipeline that remains IO-free and aligns RSU acquisition costs with HMRC guidance, while documenting the new input format and relocating tax rules documentation into a docs area.

## Goals / Non-Goals

- Goals:
  - Support Schwab JSON for transactions and awards only.
  - Preserve HMRC-compliant RSU acquisition treatment (vest date, FMV cost basis, share matching).
  - Provide dedicated documentation for Schwab JSON structures.
  - Relocate tax rules documentation into docs without breaking references.
- Non-Goals:
  - Supporting legacy Schwab CSV exports.
  - Changing CGT calculation logic beyond input normalization.

## Decisions

- Decision: Treat Schwab JSON awards as the authoritative source for vest date and FMV, and match Schwab JSON transactions to awards using a 0–7 day lookback window by symbol.
  - Alternatives considered: strict date equality only; using award grant date fields for lookup.
  - Rationale: Awards data includes explicit vest/lapse date and FMV. Schwab transactions often settle later; the current 7-day lookback preserves existing behavior while accommodating T+2 settlement. Grant dates are not the CGT acquisition date per HMRC guidance.
- Decision: Use Schwab JSON transactions `Stock Plan Activity` quantities as gross vest quantities; treat matching `Sell` rows on the same award date as tax-withheld disposals; record net shares as holdings. Emit an error when awards data is missing and RSU transactions are present.
  - Alternatives considered: using net shares deposited as the acquisition quantity; inferring withholding solely from award data.
  - Rationale: HMRC CGT acquisition cost should be based on total shares acquired at vest (employment income already taxed); withheld shares represent a disposal to fund taxes. Schwab transaction records provide explicit sell quantities for withholding that can be matched to awards; awards data remains a cross-check for total/net quantities.

## Risks / Trade-offs

- Schwab JSON may vary by account type; strict JSON parsing could reject unknown fields. Mitigation: use serde with optional/ignored fields and add validation errors pointing to missing required fields.
- Award/transaction matching could drift if brokerage changes settlement timing beyond 7 days. Mitigation: keep lookback configurable in code with clear error messaging.

## Migration Plan

1. Introduce JSON parsing for Schwab transactions and awards and remove CSV parsers.
2. Update CLI help and docs to describe JSON-only inputs.
3. Relocate tax rules doc into docs and update references in README, MCP resources, and AGENTS.
4. Update tests to use JSON fixtures and validate RSU mapping.

## Open Questions

- Confirm the intended default for 7-day lookback window (retain existing value or configure).
- Confirm whether to surface warnings when awards quantities do not reconcile with transaction totals.
