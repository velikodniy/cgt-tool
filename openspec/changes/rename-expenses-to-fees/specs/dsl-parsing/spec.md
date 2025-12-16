# DSL Parsing Spec Changes

## MODIFIED Requirements

### Requirement: Transaction Parsing

The system SHALL parse `BUY` and `SELL` transactions using the `FEES` keyword for transaction costs.

- **WHEN** a line contains `YYYY-MM-DD BUY|SELL TICKER QUANTITY @ PRICE [FEES AMOUNT]`
- **THEN** it parses as a `Buy` or `Sell` transaction with the specified date, ticker, quantity, price, and optional fees
- **AND** the fees default to 0 if not specified
- **AND** the currency of fees matches the price currency if not specified

#### Scenario: Buy with fees

- **GIVEN** line `2023-01-01 BUY AAPL 10 @ 150.00 FEES 5.00`
- **THEN** parses as Buy 10 AAPL @ 150.00 with 5.00 fees

#### Scenario: Sell with fees

- **GIVEN** line `2023-01-02 SELL AAPL 10 @ 160.00 FEES 5.00`
- **THEN** parses as Sell 10 AAPL @ 160.00 with 5.00 fees

### Requirement: Capital Return Parsing

The system SHALL parse `CAPRETURN` transactions using the `FEES` keyword for transaction costs.

- **WHEN** a line contains `YYYY-MM-DD CAPRETURN TICKER QUANTITY TOTAL VALUE FEES AMOUNT`
- **THEN** it parses as a Capital Return transaction

#### Scenario: CapReturn with fees

- **GIVEN** line `2023-01-03 CAPRETURN AAPL 100 TOTAL 500 FEES 0`
- **THEN** parses as CapReturn 100 AAPL, value 500, fees 0
