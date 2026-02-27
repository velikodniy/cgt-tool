## MODIFIED Requirements

### Requirement: Get FX Rate Tool

The system SHALL provide `get_fx_rate` tool to retrieve HMRC exchange rates.

#### Scenario: Get rate for currency and month

- **WHEN** tool is called with `currency`, `year`, and `month`
- **THEN** return rate, currency code, and period (e.g., "2024-03")

#### Scenario: Rate not found for known currency

- **WHEN** requested currency/month has no rate
- **AND** the currency exists in other cached periods
- **THEN** return error indicating the rate is missing for the requested period
- **AND** include hint that rates are available for this currency in other periods

#### Scenario: Rate not found for unknown currency

- **WHEN** requested currency/month has no rate
- **AND** the currency does not exist in any cached period
- **THEN** return error indicating the currency is unknown
- **AND** include hint about supported currencies

#### Scenario: Currency existence check is dynamic

- **WHEN** determining whether a currency is known or unknown
- **THEN** the check SHALL query actual FxCache contents
- **AND** SHALL NOT use a hardcoded year range
