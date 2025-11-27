# Feature Specification: DSL Enhancements

**Feature Branch**: `002-dsl-enhancements`
**Created**: 2025-11-27
**Status**: Draft
**Input**: User description: "Let's focus on the DSL. Let's make the following changes: 1. Add special words to commands to make them more readable: `2019-11-30 DIVIDEND GB00B3TYHH97 110.93 TAX 0` (the word TAX), `2019-05-31 CAPRETURN GB00B3TYHH97 149.75 EXPENSES 0` (the word EXPENSES), `2019-02-15 SPLIT FOO SPLIT 2` (the word SPLIT). 2. Let's use the .cgt extensions everywhere instead of .txt 3. Sort the transactions in all the test .cgt files. The transactions should go from the earliest to the latest. We should make sure that the order of dates is preserved. 4. Validate every single test case and make sure that every output in tests is correct. Download the original tests from the cgtcalc repo, our outputs should match with the original tests. We should trust them. 5. Think of DSL improvements. What could be added?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Enhanced DSL Readability (Priority: P1)

The user wants to interact with the CLI using a more natural and readable Domain Specific Language (DSL) by incorporating descriptive keywords like `TAX`, `EXPENSES`, and `SPLIT` directly into transaction commands.

**Why this priority**: Improves user experience and reduces potential for errors due to cryptic syntax.

**Independent Test**: Provide sample transaction files with the new DSL syntax. Verify successful parsing and generation of the internal representation without errors.

**Acceptance Scenarios**:

1. **Given** a DSL file containing a `DIVIDEND` command with the `TAX` keyword, **When** the file is parsed, **Then** the system correctly extracts the tax paid amount.
2. **Given** a DSL file containing a `CAPRETURN` command with the `EXPENSES` keyword, **When** the file is parsed, **Then** the system correctly extracts the expenses.
3. **Given** a DSL file containing a `SPLIT` command with the `SPLIT` keyword, **When** the file is parsed, **Then** the system correctly interprets the split ratio.

______________________________________________________________________

### User Story 2 - Robust Test Suite & Output Validation (Priority: P1)

The user requires high confidence in the accuracy of Capital Gains Tax calculations through a robust test suite where all test inputs (`.cgt` files) are chronologically sorted, and all generated outputs are meticulously validated against trusted original `cgtcalc` outputs.

**Why this priority**: Directly addresses correctness, compliance, and trust in the system's core functionality.

**Independent Test**: Re-run the entire test suite after applying sorting and format changes. Verify that all assertions pass and that generated reports match original `cgtcalc` outputs after re-conversion.

**Acceptance Scenarios**:

1. **Given** any test `.cgt` file with transactions in arbitrary order, **When** the file is processed (sorted), **Then** the internal processing order is strictly chronological.
2. **Given** all test cases, **When** reports are generated, **Then** the computed gains/losses (after accounting for precision differences) match the corresponding original `cgtcalc` output.

______________________________________________________________________

### Edge Cases

- What happens if a required keyword (e.g., `TAX` for DIVIDEND, `EXPENSES` for CAPRETURN) is missing in the new DSL?
- How does the system handle an unknown keyword being introduced into a command?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST update the DSL grammar to incorporate the `TAX` keyword for `DIVIDEND` commands: `YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_AMOUNT`.
- **FR-002**: System MUST update the DSL grammar to incorporate the `EXPENSES` keyword for `CAPRETURN` commands: `YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSE_AMOUNT`.
- **FR-003**: System MUST update the DSL grammar to incorporate the `SPLIT` keyword for `SPLIT` and `UNSPLIT` commands: `YYYY-MM-DD SPLIT FOO SPLIT RATIO` or `YYYY-MM-DD UNSPLIT FOO UNSPLIT RATIO`.
- **FR-004**: System MUST modify the parser (`parser.pest`, `parser.rs`) to correctly interpret these new DSL keywords.
- **FR-005**: System MUST ensure all internal transaction file references consistently use the `.cgt` extension.
- **FR-006**: System MUST sort transactions within all test `.cgt` files chronologically (earliest to latest date).
- **FR-007**: System MUST re-validate all test case outputs against newly downloaded original `cgtcalc` outputs, prioritizing `cgtcalc`'s results where discrepancies exist.
- **FR-008**: System MUST update `README.md` and `quickstart.md` with new DSL syntax examples.

### Key Entities

- No new entities are introduced; existing `Transaction` and `Operation` entities will adapt to new parsing logic.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All existing 21+ test cases parse successfully with the new DSL.
- **SC-002**: All test suite assertions pass after DSL changes, sorting, and output re-validation.
- **SC-003**: The DSL syntax for `DIVIDEND`, `CAPRETURN`, and `SPLIT`/`UNSPLIT` is objectively more readable as evidenced by keyword presence.
- **SC-004**: All generated test outputs (`.json`) precisely match (within `Decimal` precision) the re-derived expectations from `cgtcalc` original output data.

## Clarifications

### Session 2025-11-27

- Q: What are the primary goals for future DSL improvements? → A: Support for multiple currencies beyond GBP.

### Future Functional Requirements

- **FR-009**: System SHOULD support the definition and conversion of transactions in multiple currencies.
