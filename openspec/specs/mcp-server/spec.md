# mcp-server Specification

## Purpose

TBD - created by archiving change add-mcp-server. Update Purpose after archive.

## Requirements

### Requirement: MCP Protocol Support

The system SHALL implement an MCP server communicating via stdio using JSON-RPC 2.0.

#### Scenario: Server initialization

- **WHEN** cgt-mcp binary is started
- **THEN** it reads JSON-RPC messages from stdin
- **AND** writes JSON-RPC responses to stdout
- **AND** responds to `initialize` with server capabilities

#### Scenario: Tool discovery

- **WHEN** client sends `tools/list` request
- **THEN** server returns list of available tools with input schemas
- **AND** tool descriptions include full JSON schema documentation

#### Scenario: Resource discovery

- **WHEN** client sends `resources/list` request
- **THEN** server returns list of available documentation resources
- **NOTE** Claude Desktop requires resources to be manually added in settings

### Requirement: Parse Transactions Tool

The system SHALL provide `parse_transactions` tool to parse CGT DSL or JSON content into structured JSON.

#### Scenario: Parse valid JSON content

- **WHEN** tool is called with JSON array of transactions
- **THEN** return normalized JSON array of Transaction objects
- **AND** include all transaction fields (date, type, ticker, quantity, price, fees, currency)
- **AND** accept case-insensitive action names (BUY, buy, Buy all work)
- **AND** accept case-insensitive ticker symbols

#### Scenario: Parse valid DSL content

- **WHEN** tool is called with CGT DSL text
- **THEN** return JSON array of Transaction objects

#### Scenario: Parse invalid content

- **WHEN** tool is called with invalid content
- **THEN** return error with description of parse failure
- **AND** include hints for fixing common errors
- **AND** include examples of valid input format

#### Scenario: Validate positive amounts

- **WHEN** transaction contains zero or negative amount
- **THEN** return error explaining amounts must be positive

### Requirement: Calculate Report Tool

The system SHALL provide `calculate_report` tool to generate CGT reports.

#### Scenario: Generate report for tax year

- **WHEN** tool is called with `cgt_content` and `year` (e.g., 2024)
- **THEN** return JSON report for tax year 2024/25
- **AND** include gains, losses, disposals, matches, and pool states

#### Scenario: Report with FX conversion

- **WHEN** cgt_content contains foreign currency transactions
- **THEN** convert using bundled HMRC rates
- **AND** include both GBP and original currency in output

#### Scenario: Invalid year

- **WHEN** year parameter results in no disposals
- **THEN** return report with zero gains/losses (not an error)

### Requirement: Explain Matching Tool

The system SHALL provide `explain_matching` tool to explain how a disposal was matched.

#### Scenario: Explain Same Day match

- **WHEN** tool is called with `cgt_content`, `disposal_date`, and `ticker`
- **AND** disposal was matched by Same Day rule
- **THEN** return explanation including:
  - Matching rule applied ("Same Day")
  - Matched acquisition(s) with dates and quantities
  - Cost basis calculation
  - Resulting gain or loss

#### Scenario: Explain B&B match

- **WHEN** disposal was matched by Bed & Breakfast rule
- **THEN** return explanation including matched acquisition within 30-day window

#### Scenario: Explain S104 match

- **WHEN** disposal was matched from Section 104 pool
- **THEN** return explanation including pool state before/after and average cost used

#### Scenario: Disposal not found

- **WHEN** no disposal exists for given date and ticker
- **THEN** return error with available disposals for that ticker

### Requirement: Get FX Rate Tool

The system SHALL provide `get_fx_rate` tool to retrieve HMRC exchange rates.

#### Scenario: Get rate for currency and month

- **WHEN** tool is called with `currency`, `year`, and `month`
- **THEN** return rate, currency code, and period (e.g., "2024-03")

#### Scenario: Rate not found

- **WHEN** requested currency/month has no bundled rate
- **THEN** return error explaining rate is unavailable

### Requirement: Convert to DSL Tool

The system SHALL provide `convert_to_dsl` tool to convert JSON transactions to DSL format.

#### Scenario: Convert JSON to DSL

- **WHEN** tool is called with JSON array of transactions
- **THEN** return DSL text format compatible with cgt-tool CLI
- **AND** include currency codes in output
- **AND** include fees when present

#### Scenario: DSL output format

- **WHEN** converting BUY transaction
- **THEN** output format: `YYYY-MM-DD BUY TICKER QUANTITY @ PRICE CURRENCY [FEES AMOUNT CURRENCY]`

### Requirement: Tax Rules Resource

The system SHALL provide `tax-rules` resource with HMRC share matching documentation.

#### Scenario: Read tax rules

- **WHEN** client requests `cgt://docs/tax-rules` resource
- **THEN** return TAX_RULES.md content as text/markdown

**NOTE**: Resources require manual addition in Claude Desktop settings to be available.

### Requirement: DSL Syntax Resource

The system SHALL provide `dsl-syntax` resource with CGT DSL format documentation.

#### Scenario: Read DSL syntax

- **WHEN** client requests `cgt://docs/dsl-syntax` resource
- **THEN** return documentation of transaction format with examples

**NOTE**: Resources require manual addition in Claude Desktop settings to be available.

### Requirement: Error Handling

The system SHALL return structured errors compatible with MCP error protocol.

#### Scenario: Tool execution error

- **WHEN** a tool fails due to invalid input or calculation error
- **THEN** return JSON-RPC error with code and descriptive message
- **AND** include relevant context (line numbers, invalid values)
- **AND** include hints for fixing common errors
- **AND** include examples of valid input format

#### Scenario: Protocol error

- **WHEN** client sends malformed JSON-RPC or unknown method
- **THEN** return appropriate JSON-RPC error code (-32700, -32601, etc.)

#### Scenario: Missing currency in object format

- **WHEN** price/fees is provided as object `{"amount": "X"}` without currency field
- **THEN** return error explaining that object format requires explicit currency
- **AND** suggest using plain string for GBP or adding currency field

### Requirement: Currency Amount Format

The system SHALL support two formats for monetary amounts.

#### Scenario: GBP amount as plain string

- **WHEN** price/fees is provided as plain string (e.g., `"150"`)
- **THEN** interpret as GBP amount

#### Scenario: Foreign currency as object

- **WHEN** price/fees is provided as object with amount and currency
- **THEN** interpret using specified currency
- **AND** convert to GBP using HMRC rates for calculation

#### Scenario: Object without currency rejected

- **WHEN** price/fees is provided as object without currency field
- **THEN** return error with helpful message
- **AND** suggest using plain string for GBP

### Requirement: Graceful Shutdown

The system SHALL handle shutdown signals properly to avoid orphan processes.

#### Scenario: Stdin closed

- **WHEN** stdin is closed (e.g., Claude Desktop window closes)
- **THEN** server exits cleanly without error

#### Scenario: SIGTERM/SIGINT received

- **WHEN** server receives SIGTERM, SIGINT, or SIGHUP signal
- **THEN** server exits cleanly without error

#### Scenario: Parent process terminates

- **WHEN** parent process (Claude Desktop) terminates
- **THEN** server detects closure and exits
- **AND** does not remain running as orphan process
