## Context

`TaxPeriod` is a validated newtype over `u16` representing the start year of a UK tax year. It has two constructors:

- `new(start_year: u16) -> Result<Self, CgtError>` — validates `[1900, 2100]`.
- `from_date(date: NaiveDate) -> Self` — derives the tax year from a calendar date but skips validation.

The second constructor can produce values that violate the type's invariant.

## Goals / Non-Goals

**Goals:**

- Enforce the `[1900, 2100]` range invariant in all constructors of `TaxPeriod`.
- Maintain backward compatibility for all valid dates (the overwhelming majority of callers).

**Non-Goals:**

- Changing the valid range itself.
- Modifying the tax year derivation logic (month/day boundary).

## Decisions

**Make `from_date` call `new` internally.** The derived year is passed to `new()`, which already validates. This avoids duplicating the range check and guarantees a single source of truth for validation.

**Return `Result<Self, CgtError>` from `from_date`.** Since `new()` is fallible, `from_date` must also be fallible. Callers in `calculator.rs` already return `Result<_, CgtError>`, so propagation with `?` is straightforward.

**Alternative considered: panic on invalid input.** Rejected — the project has a strict no-panic policy.

## Risks / Trade-offs

- **API breakage within the crate**: `from_date` changes from infallible to fallible. All callers must be updated. Risk is low because there are only two call sites in `calculator.rs` and one test file, all within `cgt-core`.
- **No public crate consumers**: `cgt-core` is not published as a standalone crate, so this is an internal change with no external API impact.
