# Feature Specification: Codebase Quality Refactoring

**Feature Branch**: `009-codebase-refactoring`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "Comprehensive codebase refactoring to improve code quality, eliminate duplication, enhance error handling, and establish cleaner architecture"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Consistent Output Formatting (Priority: P1)

As a user generating CGT reports, I want all output formats (plain text, PDF) to display currency values, dates, and numbers consistently so that I can trust the accuracy and professionalism of my tax reports.

**Why this priority**: Inconsistent formatting (e.g., `-£20` vs `£-20`) directly affects user trust and report accuracy. This is visible to end users and may cause confusion or compliance issues.

**Independent Test**: Can be fully tested by generating reports in both plain and PDF formats with negative values and comparing output consistency.

**Acceptance Scenarios**:

1. **Given** a transaction resulting in a negative value, **When** I generate a plain text report, **Then** the negative sign appears in the same position as in PDF reports
2. **Given** any currency amount, **When** I generate reports in any format, **Then** thousand separators and decimal places are formatted identically
3. **Given** a date range for a tax year, **When** displayed in any format, **Then** the format is consistent (e.g., "2023/24" not "2023-2024" in one place and "Tax Year 2023/24" in another)

---

### User Story 2 - Clear Parser Error Messages (Priority: P1)

As a user entering transaction data, I want clear, actionable error messages when my input contains mistakes so that I can quickly identify and fix problems without guessing.

**Why this priority**: Current fallback to "expected COMMENT" for all errors is confusing and wastes user time. Clear errors significantly improve usability.

**Independent Test**: Can be tested by entering various malformed inputs and verifying error messages indicate the specific problem and suggest fixes.

**Acceptance Scenarios**:

1. **Given** an invalid date format in a transaction, **When** parsing fails, **Then** the error message identifies the line, shows the problematic value, and suggests valid date formats
2. **Given** a missing required field in a transaction, **When** parsing fails, **Then** the error message names the missing field
3. **Given** an unrecognized transaction type, **When** parsing fails, **Then** the error message lists valid transaction types
4. **Given** a numeric value in wrong format, **When** parsing fails, **Then** the error message shows what was expected versus what was received

---

### User Story 3 - Reliable Large Value Handling (Priority: P1)

As a user with high-value transactions, I want the system to handle large currency amounts correctly without silent truncation or data loss so that my tax calculations are accurate.

**Why this priority**: Silent conversion of large values to zero or truncation is a critical bug that could cause incorrect tax filings.

**Independent Test**: Can be tested by processing transactions with values exceeding standard integer limits and verifying calculations remain accurate.

**Acceptance Scenarios**:

1. **Given** a transaction with a value exceeding typical integer limits, **When** processed, **Then** the full value is preserved in calculations
2. **Given** an overflow scenario in currency conversion, **When** detected, **Then** a clear error is raised rather than silently returning zero
3. **Given** very large accumulated pool values, **When** calculations are performed, **Then** precision is maintained throughout

---

### User Story 4 - Configurable Tax Exemption Values (Priority: P2)

As a user filing taxes for various years, I want exemption thresholds to be easily viewable and configurable so that I can verify correctness and update values when regulations change.

**Why this priority**: Hardcoded exemption values make the system difficult to maintain and audit. Users need confidence that correct thresholds are applied.

**Independent Test**: Can be tested by viewing/modifying exemption configuration and running reports for different tax years.

**Acceptance Scenarios**:

1. **Given** a need to check exemption thresholds, **When** I examine the embedded data file in the source tree, **Then** I can see all tax year exemption values in a readable format
2. **Given** a new tax year with updated thresholds and inability to recompile, **When** I create an external TOML override file, **Then** reports for that year use the new thresholds
3. **Given** historical tax years back to the system's earliest supported year, **When** generating reports, **Then** correct exemption values are applied from embedded defaults

---

### User Story 5 - Maintainable Report Templates (Priority: P2)

As a maintainer of the CGT tool, I want report formatting to be separated from calculation logic so that I can modify report appearance without risking calculation correctness.

**Why this priority**: Current mixing of formatting and logic makes changes risky and increases testing burden. Clean separation improves maintainability.

**Independent Test**: Can be tested by modifying report template/formatting without changing core calculation code and verifying calculations remain unchanged.

**Acceptance Scenarios**:

1. **Given** a desire to change how matching rules are displayed, **When** I modify the formatting layer, **Then** calculation logic remains untouched
2. **Given** a new output format requirement, **When** adding a new formatter, **Then** I only need to implement formatting without understanding calculation internals
3. **Given** the plain text formatter, **When** examining its structure, **Then** it uses a template-based approach similar to PDF generation

---

### User Story 6 - Robust Input Validation (Priority: P2)

As a user, I want the system to validate my input data and warn me about potential issues before calculating so that I catch data entry mistakes early.

**Why this priority**: Validating invariants upfront (zero-quantity disposals, sells without holdings) prevents confusing errors during calculation.

**Independent Test**: Can be tested by submitting data with known issues and verifying appropriate warnings/errors before calculation proceeds.

**Acceptance Scenarios**:

1. **Given** a disposal with zero quantity, **When** validation runs, **Then** a clear error is raised before calculation starts
2. **Given** a sell transaction without prior matching acquisitions, **When** validation runs, **Then** a warning is issued identifying the discrepancy
3. **Given** corporate actions that would result in impossible states, **When** validation runs, **Then** the specific issue is identified

---

### User Story 7 - Efficient Processing of Large Files (Priority: P3)

As a user with extensive transaction history, I want report generation to handle large datasets efficiently so that I don't experience excessive delays or memory issues.

**Why this priority**: While not blocking, efficiency improvements benefit users with complex portfolios and enable scaling.

**Independent Test**: Can be tested by processing large transaction files and measuring memory usage and processing time.

**Acceptance Scenarios**:

1. **Given** a file with thousands of transactions, **When** generating a report, **Then** processing completes without excessive memory duplication
2. **Given** complex corporate action histories, **When** calculating cost basis adjustments, **Then** performance scales reasonably with data size

---

### Edge Cases

- What happens when currency amounts exceed 64-bit integer representation?
- How does the system handle divide-by-zero scenarios in average price calculations?
- What happens when exemption data is missing for a specific tax year?
- How are malformed dates (e.g., 31st February) handled during parsing?
- What happens when a sell occurs before any buy for a given ticker?

## Requirements *(mandatory)*

### Functional Requirements

**Formatting Consistency**

- **FR-001**: System MUST apply identical currency formatting rules across all output formats
- **FR-002**: System MUST use a single, shared formatting policy for negative numbers (sign before currency symbol)
- **FR-003**: System MUST format dates and tax year ranges consistently across all outputs
- **FR-004**: System MUST delegate all text representation (labels, headings, number display) to the output template layer

**Error Handling & Messaging**

- **FR-005**: Parser MUST provide line numbers and column positions for all syntax errors
- **FR-006**: Parser MUST identify the specific token or value that caused the error
- **FR-007**: Parser MUST suggest valid alternatives when an invalid value is detected (e.g., list valid transaction types)
- **FR-008**: System MUST raise explicit errors for overflow/underflow rather than defaulting to zero
- **FR-009**: System MUST validate input data before calculation and report all detectable issues

**Configuration & Data**

- **FR-010**: Tax exemption thresholds MUST be embedded at compile time with default values, stored in a separate data file in the source tree
- **FR-010a**: System MUST support an optional external TOML configuration file to override or extend embedded exemption values (for users who cannot recompile)
- **FR-011**: System MUST support exemption values for all tax years from 2014/15 onwards
- **FR-012**: System MUST fail gracefully with a clear message when exemption data is missing for a requested year

**Architecture & Separation**

- **FR-013**: Calculation logic MUST NOT contain user-facing text strings or formatting code
- **FR-014**: Formatters MUST receive pure data structures from the calculation layer
- **FR-015**: Plain text formatter MUST use a template-based approach for output generation
- **FR-016**: Shared utilities (formatting, validation) MUST NOT be duplicated across modules
- **FR-020**: Calculator MUST be restructured into a Matcher type with isolated passes (same-day, B&B, Section 104) with clear inputs/outputs
- **FR-021**: CAPRETURN/DIVIDEND preprocessing MUST use an acquisition ledger to avoid O(n²) recomputation

**Data Integrity**

- **FR-017**: System MUST preserve full precision for all currency calculations
- **FR-018**: System MUST guard against division by zero in all average price and ratio calculations
- **FR-019**: System MUST validate that disposals have non-zero quantities before processing

### Key Entities

- **Tax Exemption Configuration**: Collection of tax year to exemption threshold mappings; defaults embedded at compile time, optionally overridden via external TOML file
- **Formatting Policy**: Shared rules for currency, date, and number display applied uniformly across formatters
- **Validation Result**: Collection of errors and warnings discovered during input validation, with line references
- **Parser Error**: Structured error with location, message, expected values, and suggested fixes
- **Matcher**: Orchestrates CGT matching rules with isolated passes for same-day, bed-and-breakfast, and Section 104 pooling
- **Acquisition Ledger**: Per-ticker state machine tracking remaining acquisition amounts for efficient corporate action processing

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of currency values display identically between plain text and PDF outputs for the same data
- **SC-002**: 100% of parser errors include line number, problematic value, and at least one suggested fix
- **SC-003**: Zero silent data truncation or overflow - all such scenarios produce explicit errors
- **SC-004**: Tax exemption values are embedded by default and can be extended via external TOML without recompilation
- **SC-005**: No user-facing text strings exist in calculation modules
- **SC-006**: Plain text formatter uses zero inline format strings (all text in templates)
- **SC-007**: Zero duplicate utility functions across modules (single source of truth for formatting)
- **SC-008**: All existing tests continue to pass after refactoring (no regression)

## Clarifications

### Session 2025-12-09

- Q: What is the earliest supported tax year for exemption data? → A: 2014/15 (maintain current minimum)
- Q: What is the calculator refactoring scope? → A: Full extraction - create Matcher type with isolated passes (same-day, B&B, Section 104), refactor CAPRETURN/DIVIDEND preprocessing to acquisition ledger
- Q: What format should the exemption configuration file use? → A: TOML (idiomatic for Rust, simple key-value structure) as optional override; default values embedded at compile time

## Assumptions

- The standard convention for negative currency in UK tax contexts is sign before symbol (e.g., `-£20`)
- Exemption threshold data is publicly available from HMRC for historical tax years
- A template-based approach for plain text output will use an existing, well-supported template engine
- The refactoring will maintain backward compatibility with existing input file formats
- Performance improvements are secondary to correctness and maintainability

## Out of Scope

- Changes to the DSL/input file format syntax
- New transaction types or tax calculation rules
- GUI or web interface
- Multi-currency support beyond GBP
- JSON schema versioning for reports (deferred to separate feature)
- Introduction of strong newtypes (Money, Quantity) - considered but deferred as it requires extensive changes
