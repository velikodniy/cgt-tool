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

### Requirement: Schwab Transactions JSON Parser

The system SHALL parse Charles Schwab transaction JSON exports. The input is a JSON object with a `BrokerageTransactions` array containing transaction objects. Each transaction object MUST contain fields for Date, Action, Symbol, Description, Price, Quantity, Fees & Comm, and Amount.

#### Scenario: Parse basic buy transaction

- **WHEN** JSON contains a buy transaction
- **THEN** output corresponding `BUY` transaction in CGT DSL format

#### Scenario: Parse sell transaction

- **WHEN** JSON contains a sell transaction
- **THEN** output corresponding `SELL` transaction in CGT DSL format

#### Scenario: Parse dividend

- **WHEN** JSON contains a cash dividend transaction
- **THEN** output `DIVIDEND` transaction

#### Scenario: Parse dividend with tax withheld

- **WHEN** JSON contains dividend transaction followed by NRA Withholding transaction on same date for same symbol
- **THEN** combine into single `DIVIDEND` with `TAX` amount from withholding

#### Scenario: Skip non-CGT transactions

- **WHEN** JSON contains Wire Sent, Wire Received, Credit Interest, or similar cash movements
- **THEN** skip these transactions
- **AND** include count of skipped transactions in header comment

#### Scenario: Tolerate extra fields

- **WHEN** JSON objects contain extra fields beyond the expected set
- **THEN** ignore the extra fields
- **AND** proceed with parsing without error

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

### Requirement: Date Format Handling

The system SHALL parse Schwab date formats including "as of" notation.

#### Scenario: Standard date format

- **WHEN** date is `02/09/2021` or `2021-02-09`
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

- **WHEN** Schwab JSON transactions are in reverse chronological order (newest first)
- **THEN** output transactions oldest-to-newest

### Requirement: CLI Convert Subcommand

The CLI SHALL provide a `convert` subcommand with broker-specific subcommands, each defining its own arguments.

#### Scenario: Convert Schwab with positional transactions file

- **WHEN** user runs `cgt convert schwab transactions.json`
- **THEN** convert transactions JSON and output CGT DSL to stdout

#### Scenario: Convert Schwab with awards file

- **WHEN** user runs `cgt convert schwab transactions.json --awards awards.json`
- **THEN** convert transactions JSON using FMV prices from awards file
- **AND** output CGT DSL to stdout

#### Scenario: Convert without awards file when RSUs present

- **WHEN** user runs `cgt convert schwab transactions.json` (no awards)
- **AND** transactions contain Stock Plan Activity
- **THEN** exit with error explaining awards file is required

#### Scenario: Write to file

- **WHEN** user runs `cgt convert schwab transactions.json -o output.cgt`
- **THEN** write CGT DSL to specified file

#### Scenario: Show broker-specific help

- **WHEN** user runs `cgt convert schwab --help`
- **THEN** display Schwab-specific arguments (transactions file, --awards option)

### Requirement: Error Reporting

The system SHALL provide clear error messages for conversion failures.

#### Scenario: Invalid JSON format

- **WHEN** input is not valid JSON
- **THEN** error message indicates JSON parse failure

#### Scenario: Missing required fields

- **WHEN** transaction object is missing required fields (Date, Action, etc.)
- **THEN** error message identifies missing data

#### Scenario: Unparseable date

- **WHEN** date cannot be parsed
- **THEN** error message includes invalid date value

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
