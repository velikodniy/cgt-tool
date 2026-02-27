## MODIFIED Requirements

### Requirement: HMRC Rates

The system SHALL use HMRC monthly average exchange rates for the transaction month.

#### Scenario: Rate lookup

- **WHEN** looking up an FX rate
- **THEN** `FxCache::get` SHALL accept a `Currency` value, year, and month
- **AND** return the matching `RateEntry` if present
- **AND** return `None` if no rate exists for that currency/month

#### Scenario: No string normalization

- **WHEN** calling `FxCache::get`
- **THEN** no `String` allocation SHALL occur for key lookup
- **AND** the `Currency` enum value SHALL be used directly as the lookup key
