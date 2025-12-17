## MODIFIED Requirements

### Requirement: CurrencyAmount Type

The system SHALL provide a `CurrencyAmount` type that stores original amount and currency, with on-demand GBP conversion.

#### Scenario: GBP amount

- **WHEN** creating a GBP amount
- **THEN** `amount` stores the value
- **AND** `is_gbp()` returns true
- **AND** `to_gbp()` returns the amount unchanged

#### Scenario: Foreign currency amount

- **WHEN** creating a foreign currency amount
- **THEN** `amount` stores the original value
- **AND** `is_gbp()` returns false
- **AND** `to_gbp(date, fx_cache)` converts using HMRC rate for that month

#### Scenario: Currency metadata

- **WHEN** querying a `CurrencyAmount`
- **THEN** `minor_units()` returns the currency's decimal places (e.g., 2 for GBP)
- **AND** `symbol()` returns the currency symbol (e.g., "£")
- **AND** `code()` returns the ISO code (e.g., "GBP")

#### Scenario: JSON deserialization

- **WHEN** deserializing from JSON
- **THEN** only `amount` field is required
- **AND** `currency` field defaults to "GBP" if not provided

#### Scenario: JSON serialization

- **WHEN** serializing a `CurrencyAmount`
- **THEN** the output includes `amount` and `currency` only (no `gbp` field)

#### Scenario: Conversion error

- **WHEN** calling `to_gbp()` for a foreign currency
- **AND** no FX rate exists for that currency/month
- **THEN** return an error with the missing currency and month
