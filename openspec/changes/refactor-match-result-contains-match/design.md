## Context

`MatchResult` (defined in `matcher/mod.rs`) is an internal type produced by the three matching algorithms and consumed by `group_matches_into_disposals` in `calculator.rs`. It currently duplicates all five fields of the public `Match` struct (`rule`, `quantity`, `allowable_cost`, `gain_or_loss`, `acquisition_date`) alongside disposal-level metadata (`disposal_date`, `disposal_ticker`, `gross_proceeds`, `proceeds`).

The conversion from `MatchResult` to `Match` in `group_matches_into_disposals` manually maps each field, which is fragile — adding a field to `Match` requires remembering to update `MatchResult` and the mapping code.

## Goals / Non-Goals

**Goals:**

- Eliminate field duplication between `MatchResult` and `Match`
- Simplify the `MatchResult → Match` conversion to a single field extraction
- Ensure all existing tests pass without modification

**Non-Goals:**

- Changing the public `Match` or `Disposal` serialization format
- Changing matching logic or tax calculation behavior
- Making `MatchResult` itself public or serializable

## Decisions

### Embed `Match` inside `MatchResult`

`MatchResult` will contain a `Match` field named `match_detail` (the name `match` is a Rust keyword). The disposal-level fields (`disposal_date`, `disposal_ticker`, `gross_proceeds`, `proceeds`) remain directly on `MatchResult`.

**Before:**

```rust
pub struct MatchResult {
    pub disposal_date: NaiveDate,
    pub disposal_ticker: String,
    pub quantity: Decimal,
    pub gross_proceeds: Decimal,
    pub proceeds: Decimal,
    pub allowable_cost: Decimal,
    pub gain_or_loss: Decimal,
    pub rule: MatchRule,
    pub acquisition_date: Option<NaiveDate>,
}
```

**After:**

```rust
pub struct MatchResult {
    pub disposal_date: NaiveDate,
    pub disposal_ticker: String,
    pub gross_proceeds: Decimal,
    pub proceeds: Decimal,
    pub match_detail: Match,
}
```

**Rationale:** This keeps the types structurally in sync — any field added to `Match` is automatically available via `match_detail`. The conversion in `group_matches_into_disposals` becomes `m.match_detail` instead of five field copies.

**Alternative considered:** Using `Deref<Target = Match>` — rejected as it hides the relationship and complicates ownership.

## Risks / Trade-offs

- [Risk] Fields accessed as `m.quantity` must change to `m.match_detail.quantity` at all access sites → Mitigation: compiler will catch all missed sites; limited access points in `calculator.rs`.
- [Risk] Constructors in three matcher files need updating → Mitigation: mechanical change, compiler-guided.
