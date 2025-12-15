## MODIFIED Requirements

### Requirement: Schwab Awards Parser

The system SHALL parse Schwab Equity Awards in both JSON and CSV formats to obtain Fair Market Value prices and vest dates for RSU vesting events, auto-detecting the format based on file extension.

#### Scenario: Parse JSON awards format

- **WHEN** awards file has `.json` extension
- **THEN** parse as JSON with structure: `{"EquityAwards": [{"Symbol": "...", "EventDate": "MM/DD/YYYY", "FairMarketValuePrice": "$..."}]}`
- **AND** extract Symbol, EventDate (vest date), and FairMarketValuePrice from each award

#### Scenario: Parse CSV awards format

- **WHEN** awards file has `.csv` extension
- **THEN** parse as CSV with paired-row structure
- **AND** extract Symbol and Date (vest/lapse date) from transaction row
- **AND** extract FairMarketValuePrice from following award details row

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

## ADDED Requirements

### Requirement: RSU Vest Date for CGT Acquisition

The system SHALL use the RSU vest date (from awards file) as the CGT acquisition date, not the settlement date from the transactions file, per HMRC guidance CG14250 and ERSM20192.

#### Scenario: Same Day match with vest date

- **WHEN** RSU vests on 2024-01-15 (recorded in awards file)
- **AND** shares settle in brokerage on 2024-01-17 (Stock Plan Activity in transactions)
- **AND** employee sells some shares on 2024-01-15 (sale in transactions)
- **THEN** acquisition date for CGT is 2024-01-15 (vest date)
- **AND** Same Day rule matches the sale with the vest-date acquisition

#### Scenario: B&B window calculated from vest date

- **WHEN** employee sells shares on 2024-01-10
- **AND** RSU vests on 2024-02-05 (29 days after sale)
- **AND** shares settle on 2024-02-07 (31 days after sale)
- **THEN** B&B rule applies because vest date (2024-02-05) is within 30 days
- **AND** acquisition is matched to the sale at vest-date cost basis
