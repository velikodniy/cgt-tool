# Broker Conversion Spec Changes

## MODIFIED Requirements

### Requirement: Output Format

The system SHALL output DSL transactions using the `FEES` keyword.

- **WHEN** converting broker CSV records
- **THEN** the output format uses `FEES` keyword for transaction costs

#### Scenario: Standard Buy

- **GIVEN** a buy record
- **THEN** output `YYYY-MM-DD BUY TICKER QUANTITY @ PRICE CURRENCY FEES AMOUNT CURRENCY`

#### Scenario: Standard Sell

- **GIVEN** a sell record
- **THEN** output `YYYY-MM-DD SELL TICKER QUANTITY @ PRICE CURRENCY FEES AMOUNT CURRENCY`
