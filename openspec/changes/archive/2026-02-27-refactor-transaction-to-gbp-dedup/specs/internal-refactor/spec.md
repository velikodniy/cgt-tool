## ADDED Requirements

### Requirement: Operation-level GBP conversion method

`Operation<CurrencyAmount>` SHALL provide a `to_gbp` method that converts all monetary fields within the operation to GBP, returning `Operation<Decimal>`. This method encapsulates the per-variant conversion logic previously duplicated in `Transaction::to_gbp`.

#### Scenario: Behavioral equivalence with existing conversion

- **WHEN** any `Transaction` is converted via `to_gbp` using the refactored code path
- **THEN** the resulting `GbpTransaction` SHALL be identical to the result produced by the original implementation for all Operation variants (Buy, Sell, Dividend, CapReturn, Split, Unsplit)
