# Feature Specification: Plain Text Report Formatter

**Feature Branch**: `007-plain-formatter`
**Created**: 2025-12-08
**Status**: Draft
**Input**: User description: "Implement formatting with extensible architecture. Every formatter is a separate crate. CLI uses formatter crates for output formatting. JSON formatter stays in CLI. Implement 'plain' formatter using test outputs from github.com/mattjgalloway/cgtcalc. Add test outputs and verify numbers match. Update CLI args."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate Plain Text Tax Report (Priority: P1)

As a UK taxpayer, I want to generate a human-readable plain text report from my transaction data so that I can review my capital gains tax liability in a clear, structured format.

**Why this priority**: This is the core functionality - producing a readable report that matches the established cgtcalc format used by the tax community.

**Independent Test**: Can be fully tested by running the CLI with `--format plain` on any .cgt input file and comparing output structure against expected format.

**Acceptance Scenarios**:

1. **Given** a .cgt file with buy/sell transactions, **When** I run the CLI with `--format plain`, **Then** I receive a formatted plain text report with all required sections (Summary, Tax Year Details, Holdings, Transactions).

2. **Given** a .cgt file with multiple tax years of data, **When** I run the CLI with `--format plain`, **Then** the report shows each tax year separately with correct gains/losses calculated.

3. **Given** disposals using different matching rules (Same Day, Bed & Breakfast, Section 104), **When** I run the CLI with `--format plain`, **Then** each disposal shows the matching rule applied and calculation breakdown.

---

### User Story 2 - Select Output Format via CLI (Priority: P2)

As a user, I want to choose between different output formats (plain text or JSON) when running the CLI so that I can use the format most suitable for my needs.

**Why this priority**: Enables format selection, making the tool flexible for different use cases (human reading vs machine processing).

**Independent Test**: Can be tested by running the CLI with different `--format` options and verifying correct output type is produced.

**Acceptance Scenarios**:

1. **Given** the CLI, **When** I run with `--format plain`, **Then** I receive plain text output.

2. **Given** the CLI, **When** I run with `--format json`, **Then** I receive JSON output (current behavior).

3. **Given** the CLI, **When** I run without specifying format, **Then** I receive the default format (plain text).

4. **Given** the CLI, **When** I run with an unsupported format value, **Then** I receive a clear error message listing valid options.

---

### User Story 3 - Extensible Formatter Architecture (Priority: P3)

As a developer, I want the formatter system to be extensible so that new output formats can be added as separate crates without modifying the core calculator logic.

**Why this priority**: Establishes architecture for future formatters while implementing the first one.

**Independent Test**: Can be verified by examining the crate structure - the plain formatter exists as a separate crate that the CLI depends on.

**Acceptance Scenarios**:

1. **Given** the codebase structure, **When** I examine the crates, **Then** the plain formatter is a separate crate from cgt-core and cgt-cli.

2. **Given** the plain formatter crate, **When** I examine its interface, **Then** it accepts the TaxReport data model and returns formatted text.

3. **Given** the CLI crate, **When** I examine its dependencies, **Then** it imports the formatter crate to produce output.

---

### Edge Cases

- What happens when there are no disposals in a tax year? Report should show zero gains/losses.
- What happens when holdings are empty? Report should show "NONE" for holdings section.
- What happens when there are no asset events? Report should show "NONE" for asset events section.
- How are very large numbers formatted? Use standard UK currency formatting with £ symbol.
- How are negative values displayed? Show as negative numbers (e.g., £-20) for losses.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST produce plain text output matching the cgtcalc format structure (SUMMARY, TAX YEAR DETAILS, TAX RETURN INFORMATION, HOLDINGS, TRANSACTIONS, ASSET EVENTS sections).

- **FR-002**: The SUMMARY section MUST display a table with columns: Tax year, Gain, Proceeds, Exemption, Loss carry, Taxable gain, Tax (basic), Tax (higher).

- **FR-003**: The TAX YEAR DETAILS section MUST list each disposal with: quantity, ticker, date, gain/loss amount, matching rule(s) applied, and calculation breakdown.

- **FR-004**: The matching rules displayed MUST include: "SAME DAY", "BED & BREAKFAST", "SECTION 104" with relevant acquisition details.

- **FR-005**: The HOLDINGS section MUST list remaining holdings with ticker, quantity, and cost basis, or "NONE" if empty.

- **FR-006**: The TRANSACTIONS section MUST list all buy/sell transactions in reverse chronological order with date, action, quantity, ticker, price, and expenses.

- **FR-007**: The CLI MUST accept a `--format` argument with values "plain" and "json".

- **FR-008**: The CLI MUST default to "plain" format when no format is specified.

- **FR-009**: The plain formatter MUST be implemented as a separate crate (cgt-formatter-plain).

- **FR-010**: The JSON formatter MUST remain in the CLI crate (not a separate crate).

- **FR-011**: Numbers MUST be formatted with £ symbol for currency values.

- **FR-012**: The calculation breakdown MUST show the formula: `(quantity * price - expenses) - (allowable costs) = gain/loss`.

- **FR-013**: The TAX RETURN INFORMATION section MUST show per-year summary with disposals count, proceeds, allowable costs, total gains, and total losses.

### Key Entities

- **TaxReport**: The existing internal data model containing tax year summaries and holdings.
- **Disposal**: A sale event with date, ticker, quantity, proceeds, and matched acquisitions.
- **Match**: An acquisition matched to a disposal with rule type (SameDay, BedAndBreakfast, Section104).
- **Section104Holding**: Remaining holding with ticker, quantity, and total cost basis.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Plain text output structure matches cgtcalc format for all existing test cases (verified by comparing numerical values).

- **SC-002**: All test .cgt files produce correct plain text reports with matching gain/loss calculations.

- **SC-003**: CLI successfully switches between plain and JSON output formats via `--format` argument.

- **SC-004**: Plain formatter crate compiles and tests pass independently from CLI crate.

- **SC-005**: Existing JSON output functionality continues to work unchanged.
