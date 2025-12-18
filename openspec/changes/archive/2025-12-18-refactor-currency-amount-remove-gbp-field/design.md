# Design: Remove gbp field from CurrencyAmount

## Context

`CurrencyAmount` currently stores:

```rust
pub struct CurrencyAmount {
    pub amount: Decimal,    // Original amount
    pub currency: Currency, // ISO currency
    pub gbp: Decimal,       // Pre-computed GBP equivalent
}
```

The `gbp` field is computed at parse time using FX rates. This creates problems:

- Requires FX cache during parsing
- JSON deserialization requires caller to provide `gbp` value
- Allows `amount` and `gbp` to be inconsistent

## Goals

- Simplify `CurrencyAmount` to store only `amount` and `currency`
- Compute GBP equivalent on-demand when needed
- Make JSON input simpler (no `gbp` field required)
- Accept breaking change to JSON output (remove `gbp`)

## Non-Goals

- Changing how FX rates are loaded or stored
- Changing the calculation logic (same results expected)
- Changing the DSL syntax

## Decisions

### Decision 1: Remove `gbp` field, add conversion method

**What**: Replace `gbp` field with `to_gbp()` method that takes date and FX cache. Use a single constructor `new(amount, currency)` for all currencies (defaults to GBP for convenience elsewhere).

```rust
pub struct CurrencyAmount {
    pub amount: Decimal,
    pub currency: Currency,
}

impl CurrencyAmount {
    pub fn new(amount: Decimal, currency: Currency) -> Self { ... }

    /// Convert to GBP using FX rate for the given date.
    pub fn to_gbp(&self, date: NaiveDate, fx_cache: &FxCache) -> Result<Decimal, CgtError> {
        if self.currency == Currency::GBP {
            return Ok(self.amount);
        }
        let rate = fx_cache.get(&self.currency.code(), date.year(), date.month())
            .ok_or_else(|| CgtError::MissingFxRate { ... })?;
        Ok(self.amount / rate.rate_per_gbp)
    }
}
```

**Why**:

- Single source of truth (no redundant data)
- Conversion happens at point of use with correct context (date)
- Cannot have inconsistent values

**Alternatives considered**:

- Keep `gbp` as `Option<Decimal>` - Still allows inconsistency, adds complexity
- Store rate instead of GBP value - Still redundant, rates can change

### Decision 2: Remove `gbp` from JSON output

**What**: Do not emit `gbp` in serialized JSON; outputs are GBP-normalized by calculation layer.

**Why**: Simpler, avoids redundant data; v0.x allows breaking change.

### Decision 3: Reject legacy `gbp` field on input

**What**: Deserialization fails if a `gbp` field is provided. No backward compatibility.

**Why**: Prototype API; avoid silently ignoring or accepting inconsistent data.

### Decision 4: Thread FX cache through calculations

**What**: Pass `&FxCache` to matcher and calculator functions.

**Why**: Conversion needs to happen during calculation, not during parsing.

**Changes**:

- `calculate(transactions, year)` → `calculate(transactions, year, &fx_cache)`
- Matcher methods receive `&FxCache` parameter
- Parser no longer needs FX cache (returns unconverted amounts)
- Introduce GBP-normalized types (`GbpTransaction`, with generic/unchanged Operation shape) instead of mutating Transactions in place

## Risks / Trade-offs

| Risk                                       | Mitigation                                         |
| ------------------------------------------ | -------------------------------------------------- |
| Breaking change for code using `.gbp`      | Search and update all usages                       |
| Serialization needs date context           | Use wrapper type or serialize in calculation layer |
| Performance (computing GBP multiple times) | Cache in local variables during calculation        |

## Migration Plan

1. Implement `to_gbp()` and update all calculation code to use it (no `.gbp`)
2. Remove `gbp` field and related constructors
3. Simplify parser (no FX at parse time) and remove unused args from `parse_money`
4. Remove `gbp` from serialization/output; adjust formatters/MCP docs; reject legacy `gbp` input
5. Thread FX cache through calculation entry points
6. Introduce `GbpTransaction` conversion (keep Operation shape) and use it in calculation/formatting instead of mutating `Transaction`
7. Update MCP JSON input handling

## Open Questions

1. None—proceed with breaking output change (no `gbp`).
