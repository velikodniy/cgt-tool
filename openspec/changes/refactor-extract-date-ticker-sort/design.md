## Context

`cgt-core` and both formatter crates each implement date+ticker sorting inline. The closures are similar but duplicated, which creates drift risk and makes behavior changes harder to roll out consistently. This refactor is cross-crate but intentionally small: centralize one canonical ordering helper in core and reuse it at existing sort call sites.

## Goals / Non-Goals

**Goals:**

- Define one canonical ordering helper for `(date, ticker)` records in `cgt-core`.
- Replace duplicated sort closures in calculator, plain formatter, and PDF formatter.
- Preserve current observable output ordering and keep API changes minimal.

**Non-Goals:**

- Changing HMRC matching logic or tax calculations.
- Changing report schema or formatting rules.
- Introducing new dependencies or broad utility abstractions.

## Decisions

1. Create a small utility function in `cgt-core` that compares `(NaiveDate, &str)` and returns `Ordering`.

   - Rationale: this keeps logic close to domain types and allows both core and formatters to call the same implementation.
   - Alternative considered: keep closures but add comments/tests in each crate. Rejected because duplication remains and future updates can diverge.

2. Use direct call-site replacement rather than introducing a trait or generic sorting wrapper.

   - Rationale: lower complexity and minimal surface area for a single ordering rule.
   - Alternative considered: generic helper like `sort_by_date_ticker<T>(...)`. Rejected as unnecessary abstraction for the current use cases.

3. Keep sort semantics unchanged: primary sort by date ascending, secondary sort by ticker ascending.

   - Rationale: preserves existing golden outputs and avoids behavior changes.

## Risks / Trade-offs

- [Risk] A subtle ordering difference changes golden outputs unexpectedly. -> Mitigation: run full formatter and integration tests.
- [Trade-off] Formatters depend on a core utility for ordering. -> Mitigation: utility remains tiny and stable, with no new runtime dependencies.

## Migration Plan

1. Add utility and switch calculator call site.
2. Switch plain formatter and PDF formatter call sites.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test`.
4. If any ordering output changes unexpectedly, revert call-site behavior to exact previous semantics before merging.

## Open Questions

- None.
