## Context

The current formatters each compute several derived values directly from report model fields, including disposal net gain/loss totals, disposal allowable cost totals, annual gross proceeds, and taxable gain expressions. These computations are identical across plain and PDF outputs, but they are duplicated in separate crates.

This change is a small cross-module refactor touching `cgt-core`, `cgt-formatter-plain`, and `cgt-formatter-pdf`. The goal is to move shared derived calculations into model methods so formatters render values instead of re-deriving them.

## Goals / Non-Goals

**Goals:**

- Add canonical computed methods on core model types for formatter-facing derived values.
- Remove duplicated formatter calculations by replacing them with model method calls.
- Preserve existing output values and formatting behavior.

**Non-Goals:**

- No changes to matching rules, tax logic inputs, or rounding behavior.
- No DSL, parser, CLI, or output schema changes.
- No broader model redesign beyond the targeted computed methods.

## Decisions

- Add methods on `Disposal` for net gain/loss and total allowable cost aggregation from `matches`.

  - Rationale: Disposal-specific derived values belong to the disposal model and are reused by multiple renderers.
  - Alternative considered: helper functions in a formatter-shared crate; rejected because these values are intrinsic to the model, not presentation.

- Add methods on `TaxYearSummary` for annual gross proceeds and taxable gain.

  - Rationale: Year-level aggregations should be computed where year-level totals already exist, reducing repeated expressions in formatters.
  - Alternative considered: keep taxable gain as inline expression in formatters; rejected to avoid repeated business expressions.

- Keep formulas identical to existing formatter logic.

  - Rationale: issue scope is de-duplication, not behavior change.

## Risks / Trade-offs

- [Risk] Moving formulas may accidentally alter values if a field is confused (e.g., `gross_proceeds` vs `proceeds`). → Mitigation: mirror existing expressions exactly and run full tests.
- [Trade-off] Adds convenience methods to model API surface. → Mitigation: keep names explicit and scoped to existing duplicated use cases.
