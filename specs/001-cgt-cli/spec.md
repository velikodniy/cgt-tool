# Feature Specification: Capital Gains Tax (CGT) CLI Tool

**Feature Branch**: `001-cgt-cli`
**Created**: 2025-11-27
**Status**: Draft
**Input**: User description: "Build a CLI application for computing capital gain taxes. The transactions are described using a domain-specific language. The input is a file with the list of transactions and the output is a report. The format can be different: text output, JSON, PDF, etc. Let's support only JSON for now. Currently the app should support only GBP, but later it will be extended to other currencies. Also, later an LSP and MCP servers will be implemented. The DSL is simple: one transaction per line. I suggest implementing these operations: - buy transaction, - sell transaction, - capital return event, - dividend for which income tax has been taken but shares retain, - asset split - asset unsplit. An example of the buy transaction can look like: `2025-12-01 SELL SNAP 100 10.0 1.0` where 100 is the amount, 10.0 is the price and 1.0 is the expenses. It's only the idea of the command. Maybe it's reasonable to add more words in between to make the commands more readable. The transactions are processed using the FIFO principle. The CLI should support parsing only: it parses the file and produses a JSON with the transactions. It can mimic internal representation. A JSON schema should be provided. It's needed for debugging. The main command in creating a report. Given a file with transactions, the app should apply the following rules: 1. Same day trades. 2. Bed & breakfast trades where you purchase an asset within 30 days of selling the same asset. 3. Section 104 holding. See: - https://github.com/mattjgalloway/cgtcalc (don't re-use the code, use it only as a reference, use can use the test cases), - the gov.uk guides."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - DSL Parsing & Validation (Priority: P1)

The user wants to convert a raw text file containing the custom DSL into a structured, machine-readable format (JSON) to verify their data entry and debug potential issues.

**Why this priority**: This is the foundational layer. Without parsing the input correctly, no calculation can occur. It also fulfills the requirement to "support parsing only" and produce a JSON schema.

**Independent Test**: Create a file with one of every transaction type. Run the parser. Verify the output JSON matches the defined schema and contains all transaction details (date, code, amount, price, expenses).

**Acceptance Scenarios**:

1. **Given** a valid DSL file with a `BUY` and `SELL` transaction, **When** the parse command is run, **Then** the system outputs a JSON array containing both transactions with correct fields.
2. **Given** a DSL file with an invalid date format, **When** the parse command is run, **Then** the system outputs a clear error message indicating the line and nature of the error.
3. **Given** the command is run with the `--schema` flag (or similar), **When** executed, **Then** it outputs the JSON schema for the transaction format.

______________________________________________________________________

### User Story 2 - Capital Gains Report Generation (Priority: P1)

The user wants to generate a tax report from their transaction history to understand their capital gains status, applying UK CGT rules (Same Day, Bed & Breakfast, Section 104).

**Why this priority**: This is the core value proposition of the tool—automating the complex tax matching rules.

**Independent Test**: Create a transaction history that triggers all three matching rules (Same Day, B&B, Section 104). Run the report command. Verify the output JSON correctly groups these trades and calculates the gain/loss per disposal.

**Acceptance Scenarios**:

1. **Given** a history with a BUY and SELL of the same asset on the same day, **When** the report is generated, **Then** the system matches them under the "Same Day" rule.
2. **Given** a SELL followed by a BUY of the same asset within 30 days, **When** the report is generated, **Then** the system matches them under the "Bed & Breakfast" rule.
3. **Given** multiple BUYs over time followed by a partial SELL, **When** the report is generated, **Then** the system uses the Section 104 holding (average cost) to calculate the cost basis.
4. **Given** a complete history, **When** the report is generated, **Then** the output is valid JSON containing a summary of gains/losses for the tax year.

______________________________________________________________________

### Edge Cases

- What happens when a SELL quantity exceeds the current holding (short selling/data error)?
- How does the system handle transactions where price or expenses are zero?
- How does the system handle "Asset Split" and "Asset Unsplit" events regarding the cost basis of the Section 104 holding?
- What happens if the file is empty?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse the following transaction types from a text file: `BUY`, `SELL`, `DIVIDEND`, `CAPRETURN`, `SPLIT`, `UNSPLIT`.
- **FR-002**: System MUST parse lines matching the pattern: `YYYY-MM-DD ACTION TICKER AMOUNT @ PRICE [EXPENSES EXPENSE_AMOUNT]` (with support for flexible whitespace and comments).
- **FR-003**: System MUST output parsed transactions as JSON.
- **FR-004**: System MUST provide/export a JSON schema for the transaction output.
- **FR-005**: System MUST implement the "Same Day" matching rule for disposals.
- **FR-006**: System MUST implement the "Bed and Breakfast" (30-day repurchasing) matching rule.
- **FR-007**: System MUST implement "Section 104" holding logic (pooling shares and averaging cost) for all other matches.
- **FR-008**: System MUST output the final Tax Report in JSON format.
- **FR-009**: System MUST support GBP currency only (for this version).
- **FR-010**: System MUST validate that a SELL does not result in a negative holding (unless shorting is explicitly supported, assume NO for now and error).

### Key Entities

- **Transaction**: Represents a single line from the DSL (Date, Action, Ticker, Quantity, Price, Fees).
- **Holding**: Represents the current pool of shares for a specific asset (Section 104 pool), tracking total quantity and allowable cost.
- **Match**: Represents a link between a disposal (SELL) and an acquisition (BUY/Pool), categorized by rule (Same Day, B&B, S104).
- **TaxReport**: The aggregated result showing total gains/losses.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of valid sample DSL files are parsed into valid JSON (validated against schema).
- **SC-002**: The tool correctly calculates gains/losses for standard test scenarios (Same Day, B&B, S104) matching manual calculations or reference outputs (e.g., from `cgtcalc` test cases).
- **SC-003**: Parsing 1000 transactions takes less than 1 second (performance proxy for efficiency).
- **SC-004**: CLI returns specific, actionable error messages for 100% of invalid input lines (e.g., "Invalid date format on line 5").
