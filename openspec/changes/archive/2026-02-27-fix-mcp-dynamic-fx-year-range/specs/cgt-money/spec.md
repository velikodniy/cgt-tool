## ADDED Requirements

### Requirement: Currency Existence Check

The `FxCache` SHALL provide a `has_currency` method that checks whether any rate exists for a given currency code across all cached periods.

#### Scenario: Currency with rates

- **WHEN** `has_currency("USD")` is called
- **AND** the cache contains at least one USD rate entry
- **THEN** return `true`

#### Scenario: Currency without rates

- **WHEN** `has_currency("XYZ")` is called
- **AND** the cache contains no entries for XYZ
- **THEN** return `false`

#### Scenario: Case-insensitive lookup

- **WHEN** `has_currency("usd")` is called
- **AND** the cache contains USD rate entries
- **THEN** return `true`

#### Scenario: Invalid currency code

- **WHEN** `has_currency` is called with a code that is not a valid ISO 4217 currency
- **THEN** return `false`
