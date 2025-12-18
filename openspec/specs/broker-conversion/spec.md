# broker-conversion Specification

## Purpose

Define broker conversion interfaces and behaviors for turning broker exports (e.g., Schwab) into CGT DSL while remaining IO-free, deterministic, and well-validated.

## Requirements

### Requirement: Converter Trait

The system SHALL provide a `BrokerConverter` trait that broker modules implement to convert export files to CGT DSL format, with broker-specific input types.

#### Scenario: Implement converter for new broker

- **WHEN** a new broker module implements `BrokerConverter`
- **THEN** it defines an associated `Input` type for broker-specific file requirements
- **AND** it provides `convert(input) -> Result<ConvertOutput, ConvertError>` accepting the input type
- **AND** it provides `broker_name() -> &'static str` for identification

### Requirement: WASM Compatibility

The converter core SHALL operate without filesystem or network IO, accepting file contents as strings.

#### Scenario: Convert in browser environment

- **WHEN** converter is compiled to WASM
- **THEN** it accepts file contents as `String` parameters
- **AND** returns CGT DSL content as `String`
- **AND** does not require filesystem access

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

### Requirement: Date Format Handling

The system SHALL parse Schwab date formats including "as of" notation.

#### Scenario: Standard date format

- **WHEN** date is `02/09/2021`
- **THEN** parse as 2021-02-09

#### Scenario: "As of" date format (prefix)

- **WHEN** date is `as of 02/21/2021`
- **THEN** parse as 2021-02-21

#### Scenario: "As of" date format (suffix)

- **WHEN** date is `02/25/2021 as of 02/21/2021`
- **THEN** use the "as of" date `02/21/2021` (2021-02-21) as the actual transaction date

### Requirement: Output Metadata Comments

The system SHALL generate header comments in output containing conversion metadata.

#### Scenario: Standard conversion header

- **WHEN** conversion completes successfully
- **THEN** output begins with comments containing:
  - Source broker name
  - Input file names (if available)
  - Conversion timestamp
  - Count of skipped/unsupported transactions (if any)

#### Scenario: Transaction-level comments

- **WHEN** a transaction has notable characteristics (RSU vesting, "as of" date)
- **THEN** preceding comment explains the context

### Requirement: Chronological Output Order

The system SHALL output transactions in chronological order (oldest first).

#### Scenario: Reorder reversed input

- **WHEN** Schwab CSV is in reverse chronological order (newest first)
- **THEN** output transactions oldest-to-newest

### Requirement: CLI Convert Subcommand

The CLI SHALL provide a `convert` subcommand with broker-specific subcommands, each defining its own arguments.

#### Scenario: Convert Schwab with positional transactions file

- **WHEN** user runs `cgt convert schwab transactions.csv`
- **THEN** convert transactions CSV and output CGT DSL to stdout

#### Scenario: Convert Schwab with awards file

- **WHEN** user runs `cgt convert schwab transactions.csv --awards awards.json` or `--awards awards.csv`
- **THEN** convert transactions CSV using FMV prices from awards file
- **AND** auto-detect format based on file extension (.json or .csv)
- **AND** output CGT DSL to stdout

#### Scenario: Convert without awards file when RSUs present

- **WHEN** user runs `cgt convert schwab transactions.csv` (no awards)
- **AND** transactions contain Stock Plan Activity
- **THEN** exit with error explaining awards file is required

#### Scenario: Write to file

- **WHEN** user runs `cgt convert schwab transactions.csv -o output.cgt`
- **THEN** write CGT DSL to specified file

#### Scenario: Show broker-specific help

- **WHEN** user runs `cgt convert schwab --help`
- **THEN** display Schwab-specific arguments (transactions file, --awards option)

### Requirement: Error Reporting

The system SHALL provide clear error messages for conversion failures.

#### Scenario: Invalid CSV format

- **WHEN** CSV is missing required columns
- **THEN** error message lists missing columns

#### Scenario: Unparseable date

- **WHEN** date cannot be parsed
- **THEN** error message includes line number and invalid date value

#### Scenario: Unknown action type

- **WHEN** transaction has unrecognized action
- **THEN** warn and skip (do not fail entire conversion)

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
