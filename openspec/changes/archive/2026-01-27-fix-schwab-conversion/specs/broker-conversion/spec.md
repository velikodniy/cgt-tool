## MODIFIED Requirements

### Requirement: Schwab Awards Parser

The system SHALL parse Schwab Equity Awards JSON to obtain Fair Market Value prices and vest dates for RSU vesting events. The parser SHALL read the awards `Action` field and classify action types. Vesting records that provide `VestFairMarketValue` with `VestDate` SHALL be accepted, with fallback to `FairMarketValuePrice` and the parent transaction `Date` when vest-specific fields are absent. Symbol lookup SHALL be case-insensitive. Awards parsing SHALL NOT fail when a record omits `FairMarketValuePrice` if vesting FMV is provided via `VestFairMarketValue`.

#### Scenario: Parse JSON awards format

- **WHEN** awards file has `.json` extension
- **THEN** parse as JSON with structure: `{"Transactions": [{"Date": "...", "Action": "...", "Symbol": "...", "TransactionDetails": [{"Details": {"VestDate": "...", "VestFairMarketValue": "..."}}]}]}`
- **AND** accept `FairMarketValuePrice` as a fallback when `VestFairMarketValue` is not present
- **AND** extract Symbol, vest date (from `VestDate` when present, otherwise parent `Date`), and FMV from the appropriate field

#### Scenario: Non-vesting actions with empty details are accepted

- **WHEN** awards entry has `Action` of `Wire Transfer`, `Tax Withholding`, `Tax Reversal`, or `Forced Disbursement`
- **AND** `TransactionDetails` is empty
- **THEN** parsing succeeds without creating an FMV entry for that action
- **AND** later FMV lookups for other vesting events are unaffected

#### Scenario: Unknown action with empty details fails

- **WHEN** awards entry has an unrecognized `Action`
- **AND** `TransactionDetails` is empty
- **THEN** parsing returns an error indicating missing award details for that action

#### Scenario: Case-insensitive symbol lookup

- **WHEN** looking up FMV for symbol "aapl" (lowercase)
- **AND** awards file contains "AAPL" (uppercase)
- **THEN** lookup succeeds and returns the FMV for AAPL
- **AND** symbol matching is case-insensitive

#### Scenario: RSU vesting with awards file

- **WHEN** transactions JSON contains `Stock Plan Activity` for symbol on settlement date
- **AND** awards file contains an FMV for that symbol within the 7-day lookback, sourced from `VestFairMarketValue` or `FairMarketValuePrice`
- **THEN** output `BUY` transaction using the **vest date** returned by the awards lookup as the transaction date
- **AND** use the FMV from the awards file as the purchase price
- **AND** output uses the broker currency (USD) without FX conversion

#### Scenario: RSU vesting without awards file

- **WHEN** transactions JSON contains `Stock Plan Activity`
- **AND** no awards file is provided
- **THEN** return error indicating awards file is required for Stock Plan Activity

#### Scenario: FMV 7-day lookback returns vest date

- **WHEN** settlement date in transactions does not exactly match awards file
- **THEN** search up to 7 days back for matching (symbol, date) entry using vest dates
- **AND** return both the FMV and the **matched vest date** (not the query date)
- **AND** return error if no match found within lookback window

#### Scenario: Vest date used for CGT acquisition

- **WHEN** RSU vests on 2024-01-15 (recorded in awards file as `VestDate`)
- **AND** shares settle in brokerage on 2024-01-17 (Stock Plan Activity in transactions)
- **AND** employee sells some shares on 2024-01-15 (sale in transactions)
- **THEN** acquisition date for CGT is 2024-01-15 (vest date)
- **AND** Same Day rule matches the sale with the vest-date acquisition

#### Scenario: Missing FMV yields domain error

- **WHEN** Stock Plan Activity requires FMV for a symbol/date
- **AND** awards entries within lookback window lack both `VestFairMarketValue` and `FairMarketValuePrice`
- **THEN** return `MissingFairMarketValue` with symbol and date context
- **AND** do not surface a JSON parsing error for missing fields
