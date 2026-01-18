## RENAMED Requirements

- FROM: `### Requirement: Schwab Transactions CSV Parser`
- TO: `### Requirement: Schwab Transactions JSON Parser`
- FROM: `### Requirement: Schwab Awards Parser`
- TO: `### Requirement: Schwab Awards JSON Parser`

## MODIFIED Requirements

### Requirement: Schwab Transactions JSON Parser

The system SHALL parse Charles Schwab transaction JSON exports containing objects with fields: Date, Action, Symbol, Description, Quantity, Price, Fees & Comm, Amount, ItemIssueId, AcctgRuleCd. Extra fields SHALL be tolerated.

#### Scenario: Parse basic buy transaction

- **WHEN** JSON contains entry `{ "Date": "07/02/2020", "Action": "Buy", "Symbol": "VUAG", "Description": "VUAG INC", "Price": "$30.2", "Quantity": "30.5", "Fees & Comm": "$4", "Amount": "-$925.1" }`
- **THEN** output `2020-07-02 BUY VUAG 30.5 @ 30.2 USD FEES 4 USD`

#### Scenario: Parse sell transaction

- **WHEN** JSON contains entry `{ "Date": "06/06/2020", "Action": "Sell", "Symbol": "VUAG", "Description": "VUAG INC", "Price": "$33", "Quantity": "90", "Fees & Comm": "$5", "Amount": "$2965" }`
- **THEN** output `2020-06-06 SELL VUAG 90 @ 33 USD FEES 5 USD`

#### Scenario: Parse dividend

- **WHEN** JSON contains entry `{ "Date": "03/04/2021", "Action": "Cash Dividend", "Symbol": "BNDX", "Description": "VANGUARD...", "Amount": "$3.65" }`
- **THEN** output `2021-03-04 DIVIDEND BNDX 0 TOTAL 3.65 USD TAX 0 USD`

#### Scenario: Parse dividend with tax withheld

- **WHEN** JSON contains dividend entry followed by NRA Withholding entry on same date for same symbol
- **THEN** combine into single `DIVIDEND` with `TAX` amount from withholding

#### Scenario: Skip non-CGT transactions

- **WHEN** JSON contains Wire Sent, Wire Received, Credit Interest, or similar cash movements
- **THEN** skip these entries
- **AND** include count of skipped transactions in header comment

#### Scenario: Tolerate extra fields

- **WHEN** JSON contains extra keys beyond the expected set
- **THEN** ignore the extra keys
- **AND** proceed with parsing without error

### Requirement: Schwab Awards JSON Parser

The system SHALL parse Schwab Equity Awards JSON format only to obtain Fair Market Value prices and vest dates for RSU vesting events. Symbol lookup SHALL be case-insensitive.

#### Scenario: Parse JSON awards format

- **WHEN** awards file is JSON with structure: `{ "Transactions": [ { "Date": "MM/DD/YYYY", "Symbol": "...", "TransactionDetails": [ { "Details": { "FairMarketValuePrice": "$...", "NetSharesDeposited": "...", "SharesSoldWithheldForTaxes": "..." } } ] } ] }`
- **THEN** extract Symbol, vest Date, FairMarketValuePrice, NetSharesDeposited, and SharesSoldWithheldForTaxes

#### Scenario: Case-insensitive symbol lookup

- **WHEN** looking up FMV for symbol "aapl" (lowercase)
- **AND** awards file contains "AAPL" (uppercase)
- **THEN** lookup succeeds and returns the FMV for AAPL
- **AND** symbol matching is case-insensitive

#### Scenario: RSU vesting with awards file

- **WHEN** transactions JSON contains `Stock Plan Activity` for symbol on settlement date
- **AND** awards file contains FMV price for that symbol within 7-day lookback
- **THEN** output `BUY` transaction using the vest date from awards file as the transaction date
- **AND** use the FMV price from awards file as the purchase price
- **AND** use the Stock Plan Activity quantity as the acquisition quantity

#### Scenario: RSU vesting without awards file

- **WHEN** transactions JSON contains `Stock Plan Activity`
- **AND** no awards file is provided
- **THEN** return error indicating awards file is required for Stock Plan Activity

#### Scenario: FMV 7-day lookback returns vest date

- **WHEN** settlement date in transactions does not exactly match awards file
- **THEN** search up to 7 days back for matching (symbol, date) entry
- **AND** return both the FMV and the matched vest date
- **AND** return error if no match found within lookback window

#### Scenario: Tax-withholding sell matching

- **WHEN** awards file shows SharesSoldWithheldForTaxes on vest date
- **AND** transactions JSON contains Sell entries on settlement date totaling the withheld quantity
- **THEN** interpret the Sell entries as withholding disposals linked to that vest
