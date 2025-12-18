## MODIFIED Requirements

### Requirement: Transaction Types

The system SHALL parse: BUY, SELL, DIVIDEND, CAPRETURN, SPLIT, UNSPLIT.

#### Scenario: Parse BUY/SELL

- **WHEN** a line contains `YYYY-MM-DD BUY|SELL TICKER QUANTITY @ PRICE [FEES AMOUNT]`
- **THEN** extract all fields into a transaction record
- **AND** default FEES to 0 when omitted

#### Scenario: Parse DIVIDEND

- **WHEN** a line contains `YYYY-MM-DD DIVIDEND TICKER QUANTITY TOTAL VALUE [TAX AMOUNT]`
- **THEN** extract dividend details including optional tax withheld
- **AND** default TAX to 0 when omitted

#### Scenario: Parse CAPRETURN

- **WHEN** a line contains `YYYY-MM-DD CAPRETURN TICKER QUANTITY TOTAL VALUE [FEES AMOUNT]`
- **THEN** extract capital return details
- **AND** default FEES to 0 when omitted

#### Scenario: Parse SPLIT/UNSPLIT

- **WHEN** a line contains `YYYY-MM-DD SPLIT|UNSPLIT TICKER RATIO VALUE`
- **THEN** extract corporate action with ratio
