## Context

`Transaction::to_gbp` in `crates/cgt-core/src/models.rs` converts a transaction's monetary fields from their original currency to GBP. The current implementation matches on each `Operation` variant individually, repeating the same `amount_to_gbp` conversion pattern across Buy, Sell, Dividend, and CapReturn arms (~70 lines). Split and Unsplit arms are trivial copies.

## Goals / Non-Goals

**Goals:**

- Move the per-variant conversion logic into `Operation<CurrencyAmount>::to_gbp()` so each variant's conversion is defined once
- Reduce `Transaction::to_gbp` to a thin wrapper that delegates to the operation's method

**Non-Goals:**

- Changing the conversion logic or error handling
- Modifying the `Operation` enum's structure or public API
- Changing the `amount_to_gbp` helper function

## Decisions

**Add `to_gbp` method on `Operation<CurrencyAmount>`**

The new method has signature:

```rust
fn to_gbp(&self, date: NaiveDate, fx_cache: Option<&FxCache>) -> Result<Operation<Decimal>, CgtError>
```

This is a private method (not `pub`) since the conversion is only used by `Transaction::to_gbp`. The match logic moves verbatim from `Transaction::to_gbp` into this method, and `Transaction::to_gbp` becomes a simple delegation.

**Alternative considered**: A generic `map_amounts` method on `Operation<CurrencyAmount>` that takes a closure for converting each monetary field. Rejected because the conversion needs `date` and `fx_cache` context, making the closure signature awkward, and there is currently no other use case for mapping amounts.

## Risks / Trade-offs

- [Regression risk] Mitigated by the existing comprehensive test suite. All golden-file tests must pass without modification.
- [Minimal code motion] The match arms move without modification, reducing the chance of introducing errors during refactoring.
