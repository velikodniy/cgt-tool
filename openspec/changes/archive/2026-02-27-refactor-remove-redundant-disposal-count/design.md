## Context

`TaxYearSummary` stores `disposal_count: u32` as a field set during construction in `calculator.rs`. It always equals `disposals.len() as u32`. Two construction sites (`build_tax_year_summary` and `build_all_tax_year_summaries`) both compute and store it. Formatters and serialization read it as a field.

## Goals / Non-Goals

**Goals:**

- Remove the redundant `disposal_count` field from `TaxYearSummary`
- Add a `disposal_count()` method deriving the value from `disposals.len()`
- Maintain JSON serialization compatibility so `disposal_count` still appears in output
- Zero behavioral change in calculation logic or output

**Non-Goals:**

- Changing the spec definition of disposal count semantics
- Modifying any tax calculation logic

## Decisions

1. **Method over computed-on-access field**: Use `pub fn disposal_count(&self) -> u32` rather than a getter trait or lazy field. `Vec::len()` is O(1), there is no caching benefit, and a method is the simplest Rust idiom for derived data.

2. **Custom Serialize impl**: Remove `#[derive(Serialize)]` and implement `Serialize` manually for `TaxYearSummary` to include `disposal_count` as a virtual field in JSON output. This preserves the exact field order and values in serialized output, avoiding any golden file changes.

3. **Deserialization**: `Deserialize` remains derived. Old JSON files with `disposal_count` will still deserialize since unknown fields are silently ignored by default.

## Risks / Trade-offs

- [Risk] Custom `Serialize` impl adds maintenance burden -> Mitigated by the impl being straightforward and well-tested via golden file comparisons
- [Risk] Golden files contain `disposal_count` field -> Serialization preserves it, so no golden file changes needed
