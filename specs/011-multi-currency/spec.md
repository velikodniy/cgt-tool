# Feature Specification: Multi-Currency FX Conversion

**Feature Branch**: `011-multi-currency`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "Support multiple currencies. The reports should still contain GBP but we can add the original sum in parenthesis. All the calculations should be done in GBP. We need to update the syntax to support currency codes (GBP is the default). Conversion should be done using the monthly rates from https://www.trade-tariff.service.gov.uk/exchange_rates. The tool can be shipped with the latest rates but it also should support loading rates from a file downloaded from the website. Use XML as the main format. Create a separate crate for conversion. The CLI should have an arg to specify the folder with .xml files. Use efficient data structures."

## Clarifications

### Session 2025-12-09

- Q: What rounding/precision should be used for FX conversion and display? → A: Use 6dp internally; display GBP to 2dp and each original currency using its standard minor units (e.g., JPY 0dp, USD/EUR 2dp).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert non-GBP inputs to GBP with dual display (Priority: P1)

Users can enter transactions in multiple currencies and see calculations in GBP while reports also show the original amount and currency in parentheses.

**Why this priority**: Ensures compliant GBP calculations while preserving original currency context for audit—core to correctness and trust.

**Independent Test**: Provide FX XML for EUR/USD, process sample transactions dated in those months, and confirm reports show GBP primary values plus original currency in parentheses with correct conversions.

**Acceptance Scenarios**:

1. **Given** a transaction in EUR dated within a month that has an FX rate, **When** the report is generated, **Then** the gain/loss is calculated in GBP using that month’s EUR rate and the line shows the original EUR amount in parentheses.
2. **Given** a transaction in GBP without an explicit currency code, **When** the report is generated, **Then** it is treated as GBP by default and no conversion is applied.

---

### User Story 2 - Load FX rates from provided XML folder with safe fallback (Priority: P2)

Operators can point the CLI to a folder of monthly FX XML files; when a needed rate is missing or the folder is absent, the tool falls back to bundled latest rates and surfaces any gaps.

**Why this priority**: Keeps rates current without blocking processing when local files are incomplete, preserving continuity.

**Independent Test**: Run CLI with `--fx-folder` pointing to a folder containing one month of rates; process transactions across two months; verify the present month uses provided rates, missing month uses bundled rates, and a clear warning notes the fallback.

**Acceptance Scenarios**:

1. **Given** a valid FX XML for March 2025 in the folder, **When** processing a March 2025 transaction, **Then** the March rate from that file is used.
2. **Given** no FX XML for April 2025 in the folder, **When** processing an April 2025 transaction, **Then** the bundled rate for April 2025 is used and the output notes the fallback.

---

### User Story 3 - Currency-aware input syntax with validation (Priority: P3)

Users can specify currency codes per transaction; unsupported codes are rejected with clear guidance while GBP remains the default when omitted.

**Why this priority**: Prevents silent errors from mistyped currency codes and keeps existing GBP flows intact.

**Independent Test**: Provide inputs with explicit USD, EUR, and an invalid code; verify valid codes process with conversion, invalid code fails with a clear message, and GBP-only lines continue to work.

**Acceptance Scenarios**:

1. **Given** a transaction line with `USD`, **When** processing, **Then** it is accepted, converted using the correct monthly rate, and reported with both GBP and USD values.
2. **Given** a transaction line with an unknown code (e.g., `ZZZ`), **When** processing, **Then** the tool rejects the input with a clear error pointing to supported codes.

---

### Edge Cases

- What happens when a transaction month has no rate in provided XML and no bundled rate exists? → Processing stops for that transaction with a clear, actionable error listing the missing currency/month.
- How does the system handle multiple rate files containing the same month/currency? → The latest file by timestamp or highest priority source is used; duplicates are ignored with a warning.
- How are historical transactions dated outside available rate coverage handled? → They are rejected with guidance to supply the needed historical rates.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST accept currency codes per transaction line, defaulting to GBP when omitted, and reject unsupported codes with a clear message listing valid options.
- **FR-002**: The system MUST convert all monetary amounts to GBP for calculations using the monthly FX rate matching the transaction’s calendar month and currency.
- **FR-003**: Reports MUST display GBP amounts as the primary value and include the original amount and currency in parentheses for any non-GBP transaction.
- **FR-004**: The CLI MUST allow specifying a folder containing monthly FX XML files; when absent or incomplete, the system MUST fall back to bundled latest rates and surface any gaps.
- **FR-005**: If no applicable rate exists for a transaction’s currency/month in either provided or bundled data, the system MUST fail that transaction with a precise error identifying the missing currency/month and how to supply it.
- **FR-006**: The system MUST parse and cache FX rates to allow efficient lookup across large batches without re-reading XML per transaction.
- **FR-007**: The system MUST log or report which FX source (provided folder vs bundled) and rate month were used for each non-GBP transaction to support auditability.
- **FR-008**: The system MUST maintain calculation consistency: all intermediate and final taxable calculations are in GBP, unaffected by display of original amounts.
- **FR-009**: The system MUST use 6dp internal precision for FX conversion, display GBP to 2dp, and display original currencies using their standard minor units (e.g., JPY 0dp, USD/EUR 2dp).

### Key Entities *(include if feature involves data)*

- **Currency Amount**: A monetary value paired with a currency code (original input); may be converted to GBP for calculations while retaining the original for display.
- **Monthly FX Rate**: A rate for a specific currency and calendar month, sourced from provided XML or bundled data; used to convert original amounts to GBP.
- **FX Rate Source**: Identifies whether a rate came from user-provided XML or bundled defaults, including file/month metadata for audit.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of non-GBP transactions are converted using the correct monthly rate for their currency and month, with no silent fallbacks.
- **SC-002**: Reports show original currency amounts in parentheses for 100% of non-GBP transactions while keeping GBP as the primary figure.
- **SC-003**: When rates are missing for a currency/month, users receive a clear error or warning identifying the gap and how to resolve it in at most one run.
- **SC-004**: Processing 10,000 transactions with mixed currencies completes without noticeable delay to the user (under 2 minutes end-to-end on a typical workstation) and without repeated XML reads.
- **SC-005**: CLI fx folder option is recognized and used in 100% of runs where provided, with bundled rates used only when a required month/currency is absent from the folder.
