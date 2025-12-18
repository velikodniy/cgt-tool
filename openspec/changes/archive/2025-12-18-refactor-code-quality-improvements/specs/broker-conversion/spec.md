## MODIFIED Requirements

### Requirement: Schwab Awards Parser

The system SHALL parse Schwab Equity Awards in both JSON and CSV formats to obtain Fair Market Value prices and vest dates for RSU vesting events, auto-detecting the format based on file extension. Symbol lookup SHALL be case-insensitive.

#### Scenario: Parse JSON awards format

- **WHEN** awards file has `.json` extension
- **THEN** parse as JSON with structure: `{"EquityAwards": [{"Symbol": "...", "EventDate": "MM/DD/YYYY", "FairMarketValuePrice": "$..."}]}`
- **AND** extract Symbol, EventDate (vest date), and FairMarketValuePrice from each award

#### Scenario: Parse CSV awards format

- **WHEN** awards file has `.csv` extension
- **THEN** parse as CSV with paired-row structure
- **AND** extract Symbol and Date (vest/lapse date) from transaction row
- **AND** extract FairMarketValuePrice from following award details row

#### Scenario: Case-insensitive symbol lookup

- **WHEN** looking up FMV for symbol "aapl" (lowercase)
- **AND** awards file contains "AAPL" (uppercase)
- **THEN** lookup succeeds and returns the FMV for AAPL
- **AND** symbol matching is case-insensitive

#### Scenario: CSV paired-row structure

- **WHEN** CSV contains transaction row with Action="Lapse" and Date filled
- **AND** next row has empty Date/Symbol but filled FairMarketValuePrice
- **THEN** key the FMV and vest date by (Symbol, Date) from the transaction row

#### Scenario: RSU vesting with awards file

- **WHEN** transactions CSV contains `Stock Plan Activity` for symbol on settlement date
- **AND** awards file contains FMV price for that symbol within 7-day lookback
- **THEN** output `BUY` transaction using the **vest date from awards file** as the transaction date
- **AND** use the FMV price from awards file as the purchase price

#### Scenario: RSU vesting without awards file

- **WHEN** transactions CSV contains `Stock Plan Activity`
- **AND** no awards file is provided
- **THEN** return error indicating awards file is required for Stock Plan Activity

#### Scenario: FMV 7-day lookback returns vest date

- **WHEN** settlement date in transactions does not exactly match awards file
- **THEN** search up to 7 days back for matching (symbol, date) entry
- **AND** return both the FMV and the **matched vest date** (not the query date)
- **AND** return error if no match found within lookback window

#### Scenario: Vest date used for CGT acquisition

- **WHEN** RSU vests on 15th, shares settle on 17th, and sell occurs on 15th
- **THEN** acquisition date is 15th (vest date, not 17th settlement date)
- **AND** Same Day rule matches the sale with the acquisition

### Requirement: Schwab Transactions CSV Parser

The system SHALL parse Charles Schwab transaction CSV exports containing columns: Date, Action, Symbol, Description, Price, Quantity, Fees & Comm, Amount. Extra columns with empty values SHALL be tolerated.

#### Scenario: Parse basic buy transaction

- **WHEN** CSV contains row `07/02/2020,Buy,VUAG,VUAG INC,$30.2,30.5,$4,-$925.1`
- **THEN** output `2020-07-02 BUY VUAG 30.5 @ 30.2 USD FEES 4 USD`

#### Scenario: Parse sell transaction

- **WHEN** CSV contains row `06/06/2020,Sell,VUAG,VUAG INC,$33,90,$5,$2965`
- **THEN** output `2020-06-06 SELL VUAG 90 @ 33 USD FEES 5 USD`

#### Scenario: Parse dividend

- **WHEN** CSV contains row `03/04/2021,Cash Dividend,BNDX,VANGUARD...,,,,$3.65`
- **THEN** output `2021-03-04 DIVIDEND BNDX 0 TOTAL 3.65 USD TAX 0 USD`

#### Scenario: Parse dividend with tax withheld

- **WHEN** CSV contains dividend row followed by NRA Withholding row on same date for same symbol
- **THEN** combine into single `DIVIDEND` with `TAX` amount from withholding

#### Scenario: Skip non-CGT transactions

- **WHEN** CSV contains Wire Sent, Wire Received, Credit Interest, or similar cash movements
- **THEN** skip these rows
- **AND** include count of skipped transactions in header comment

#### Scenario: Tolerate extra empty columns

- **WHEN** CSV contains extra columns beyond the expected set
- **AND** those extra columns have empty values
- **THEN** ignore the extra columns
- **AND** proceed with parsing without error
