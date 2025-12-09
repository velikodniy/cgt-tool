# Feature Specification: Better Testing Coverage

**Feature Branch**: `010-better-testing`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "Let's focus on better testing. Use the items above to start a new feature branch"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validate 2024/25 rate split (Priority: P1)

A tax reviewer can run the calculator against scenarios spanning 29–30 Oct 2024 and see gains taxed at pre-change rates before the cutoff and post-change rates after, per HMRC guidance.

**Why this priority**: Incorrect rate splits would mis-state tax due for 2024/25; highest compliance risk.

**Independent Test**: Provide one input file with disposals on both sides of 30 Oct 2024 and confirm tax-year summary shows distinct rate bands and correct totals.

**Acceptance Scenarios**:

1. **Given** disposals on 29 Oct 2024 and 30 Oct 2024, **When** the report is generated, **Then** gains before 30 Oct use old main rates and gains on/after 30 Oct use 18%/24% main rates.
2. **Given** a residential property disposal on 30 Oct 2024, **When** the report is generated, **Then** residential rates remain unchanged while main-rate assets split correctly.

---

### User Story 2 - Preserve correct pools and gains with dividends, equalisation, expenses (Priority: P2)

A user can include accumulation dividends, CAPRETURN equalisation payments, and transaction expenses, and the tool produces gains/losses and pools that match HMRC rules.

**Why this priority**: These adjustments directly affect allowable costs; wrong handling creates over/understated gains.

**Independent Test**: Run fixtures that include accumulation dividends across tax years, equalisation payments without share counts, and buys/sells with stamp duty/fees; compare outputs to expected gains and pool balances.

**Acceptance Scenarios**:

1. **Given** an accumulation dividend after a partial disposal, **When** the scenario is processed, **Then** only shares held on the dividend date adjust the pool and prior disposals stay unchanged.
2. **Given** a CAPRETURN payment with no quantity provided, **When** the scenario is processed, **Then** the Section 104 pool cost is reduced by the lump sum on that date.
3. **Given** buys and sells with fees and stamp duty, **When** the summary is produced, **Then** proceeds exclude fees, allowable costs include fees/duty, and rounding is applied only at final presentation.

---

### User Story 3 - Reporting clarity in text output (Priority: P3)

A user can review the plain-text report and see B&B quantities, expenses, rate splits, and dividends that align with the underlying calculations, without mismatches or missing fields.

**Why this priority**: Report discrepancies erode trust and make reviews harder; clarity in the primary text output keeps audits straightforward.

**Independent Test**: Generate the text output for a mixed scenario and verify matched quantities, totals, and narrative sections align with expected calculations.

**Acceptance Scenarios**:

1. **Given** a B&B match and Section 104 pool, **When** the text report is generated, **Then** it shows the matched quantities and costs consistent with calculations.
2. **Given** a scenario with dividends, expenses, and rate splits, **When** the text report is generated, **Then** totals and per-event lines match the computed values in the same report.

---

### Edge Cases

- Boundary disposals on 29/10/2024 vs 30/10/2024 and mixed asset types (main vs residential) in one file.
- Accumulation dividends received after full disposal of an asset (should not error and should not alter prior gains).
- Multiple CAPRETURN payments across tax years reducing pool cost sequentially.
- Inputs with tabs or extra spaces in DIVIDEND lines still parse as four fields.
- High-precision prices/fees that could introduce rounding drift when summed across transactions.
- Sell-to-cover lines intentionally omitted or marked to avoid double-counting.
- Currency-coded amounts (e.g., USD) should be rejected or flagged explicitly until FX support exists.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Add regression tests covering 2024/25 disposals before and on/after 30 Oct 2024, asserting main-rate changes (10/20 → 18/24) and unchanged residential rates per HMRC policy paper dated 6 Nov 2024.
- **FR-002**: Add tests confirming accumulation dividends adjust only holdings present on the dividend date and do not retroactively change gains for already disposed shares.
- **FR-003**: Add tests allowing dividends after full disposal without hard failure and ensuring pool adjustments are skipped when holdings are zero.
- **FR-004**: Add tests for CAPRETURN equalisation that reduce Section 104 pool cost by the lump sum without requiring a unit count, including multiple payments over time.
- **FR-005**: Add tests ensuring proceeds exclude fees, allowable costs include fees and stamp duty, and rounding is deferred until final outputs to avoid drift.
- **FR-006**: Add a regression test where B&B reporting shows the matched quantity (not full pool) consistently between calculation and narrative in the text output.
- **FR-007**: Add a guardrail test that clearly reports unsupported non-GBP amounts until FX conversion is implemented.
- **FR-008**: Add report-level assertions ensuring the plain-text output reflects computed matched quantities, proceeds, costs, and totals for a shared fixture.
- **FR-009**: Document test fixtures and expected outputs for these cases to guide future maintenance and prevent silent regressions.

### Key Entities

- **CGT Test Fixture**: A curated input file representing real-world transaction mixes (dates, asset types, dividends, fees, equalisation, FX-coded values) used across text assertions.
- **Tax Period Banding**: The mapping of transaction dates to applicable CGT rate bands (pre- vs post-30 Oct 2024) used when validating reported tax calculations.

### Assumptions

- HMRC main-rate changes effective 30 Oct 2024 (per 6 Nov 2024 policy paper) are the authoritative reference for rate split expectations.
- FX conversion is not yet supported; current behavior is to reject/flag non-GBP inputs.
- Validation will rely on plain-text output; cross-format parity is out of scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: New automated tests cover all listed scenarios (FR-001 through FR-008) and pass in CI.
- **SC-002**: A mixed 29/30 Oct 2024 fixture produces rate-split outputs that match HMRC main-rate changes with zero mismatches in expected vs actual totals.
- **SC-003**: Fixtures with dividends, CAPRETURN, and expenses yield gains within £0.01 of expected calculations and no parser rejections for valid whitespace variations.
- **SC-004**: The plain-text report for a mixed fixture shows no discrepancies between reported matched quantities, proceeds, costs, or totals and the expected values.
- **SC-005**: FX-coded inputs trigger a clear unsupported-currency message in tests 100% of the time until conversion support is added.
