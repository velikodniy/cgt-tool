# Feature Specification: Internal Data Model Improvements

**Feature Branch**: `005-internal-data-model`
**Created**: 2025-12-08
**Status**: Draft
**Input**: User description: "focus on the internal format (reflected in the JSON files in tests/). Review it carefully, propose improvements, make it closer to the domain. Take into account the project goals. Note that in the future we'll also add formatters to generate human-readable reports. Maybe it should be reflected in the internal data model. Also, check if some fields are really necessary and it's logical to keep them (e.g. `tax_year`)"

## Problem Statement

The current internal data model has several issues that impact usability, domain clarity, and future extensibility:

1. **Ambiguous `tax_year` field**: The top-level `tax_year` field (e.g., `"tax_year": 2018`) is misleading because:

   - It shows the start year, not the full tax year range (2018/19)
   - The matches array contains disposals spanning multiple tax years
   - UK tax years run April 6 to April 5, so "2018" could be confusing

2. **Missing disposal context**: Each `Match` lacks information about which disposal (sale) it relates to, making it difficult to:

   - Generate human-readable reports that show "Sale of X shares on Y date"
   - Track partial disposals from a single sale
   - Understand the original transaction that triggered the match

3. **Imprecise decimal representations**: Values like `"proceeds": "34.202000000000005"` expose floating-point artifacts and lack currency context

4. **Limited match information**: The `Match` struct doesn't capture:

   - Which acquisition(s) were matched
   - Whether the match was partial or complete
   - The acquisition date (important for B&B rule explanation)

5. **Flat structure limits report formatting**: The current structure doesn't naturally support:

   - Grouping by tax year for multi-year reports
   - Grouping by asset for portfolio views
   - Showing disposal-by-disposal breakdowns

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate Human-Readable Tax Report (Priority: P1)

As a taxpayer, I want to generate a clear, human-readable tax report from my transaction data so I can understand my capital gains obligations and complete my self-assessment.

**Why this priority**: This is the core value proposition - users need to understand their tax position. The internal data model must support clear report generation.

**Independent Test**: Can generate a report that shows each disposal with its matched acquisitions, gains/losses, and tax year totals that a taxpayer can use for self-assessment.

**Acceptance Scenarios**:

1. **Given** a data model containing disposal matches, **When** I request a report for tax year 2023/24, **Then** I see only disposals within 6 April 2023 to 5 April 2024 with their gain/loss calculations
2. **Given** a disposal matched using B&B rule, **When** I view the report, **Then** I see both the disposal date and the matched acquisition date (within 30 days)
3. **Given** multiple assets sold in a tax year, **When** I view the report, **Then** I see a summary with total gains, total losses, and net position

---

### User Story 2 - Understand Matching Logic (Priority: P2)

As a taxpayer reviewing my CGT calculation, I want to understand how each sale was matched with purchases so I can verify the calculator applied HMRC rules correctly.

**Why this priority**: Transparency in calculations builds trust and helps users verify correctness.

**Independent Test**: Can trace any disposal back to its matched acquisition(s) with clear rule attribution.

**Acceptance Scenarios**:

1. **Given** a same-day match, **When** I view the disposal details, **Then** I see the same-day acquisition with quantities and costs
2. **Given** a B&B match, **When** I view the disposal details, **Then** I see the acquisition date (1-30 days after disposal) and the rule explanation
3. **Given** a Section 104 pool match, **When** I view the disposal details, **Then** I see the pool's average cost per share at disposal time

---

### User Story 3 - Multi-Year Portfolio View (Priority: P3)

As an investor with transactions spanning multiple tax years, I want to see my complete transaction history organized by tax year so I can review my CGT position over time.

**Why this priority**: Supports long-term planning and historical review, but not critical for immediate tax filing.

**Independent Test**: Can display transactions and gains/losses grouped by UK tax year with running totals.

**Acceptance Scenarios**:

1. **Given** transactions spanning 2022/23 and 2023/24, **When** I request a multi-year view, **Then** I see separate sections per tax year with appropriate summaries
2. **Given** carried forward losses, **When** I view subsequent years, **Then** I see how losses offset gains

---

### Edge Cases

- What happens when a single sale is matched across multiple rules (part same-day, part B&B, part S104)?
- How does the model handle zero-cost acquisitions (bonus shares)?
- How are foreign currency transactions represented (out of scope for initial version)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Data model MUST represent disposals as the primary organizing concept, with matches as children of disposals
- **FR-002**: Each disposal MUST include the original sale date, ticker, total quantity sold, and gross proceeds
- **FR-003**: Each match MUST identify the matching rule applied (Same Day, B&B, Section 104)
- **FR-004**: B&B matches MUST include the acquisition date that was matched
- **FR-005**: Section 104 matches MUST include the pool's average cost per share at time of disposal
- **FR-006**: Monetary values MUST be represented with consistent precision (2 decimal places for display, higher precision internally)
- **FR-007**: Data model MUST support grouping disposals by UK tax year (April 6 to April 5)
- **FR-008**: Tax year representation MUST use a validated TaxPeriod type that serializes to standard UK notation (e.g., "2023/24") and rejects invalid year combinations (e.g., "2023/27")
- **FR-009**: Data model MUST provide summary totals: total gains, total losses, net gain/loss per tax year
- **FR-010**: Holdings (Section 104 pools) MUST be included showing end-of-period positions

### Key Entities

- **Disposal**: A sale event that triggers CGT calculation. Contains date, ticker, quantity, proceeds, and one or more matches.
- **Match**: How a disposal (or portion) was matched to an acquisition. Includes matching rule, quantity matched, allowable cost, and gain/loss.
- **TaxPeriod**: A validated UK tax year identifier ensuring consecutive years (e.g., "2023/24"). Rejects invalid combinations like "2023/27". Serializes to "YYYY/YY" format.
- **TaxYearSummary**: A UK tax year period (April 6 to April 5) identified by a TaxPeriod, containing disposals and summary totals.
- **Section104Pool**: The pooled holding for a security showing quantity and total cost basis.
- **TaxReport**: The complete output containing one or more tax years, disposals, and end-of-period holdings.

## Proposed Data Model Changes

### Current Structure Issues

```text
TaxReport
├── tax_year: i32          // Ambiguous - which tax year? Start year only
├── matches: Vec<Match>    // Flat list loses disposal context
├── total_gain/loss/net    // Only covers one unclear period
└── holdings: Vec<...>
```

### Recommended Structure

```text
TaxReport
├── tax_years: Vec<TaxYearSummary>    // Multi-year support
│   ├── period: TaxPeriod              // Validated type, serializes to "2023/24"
│   ├── disposals: Vec<Disposal>       // Organized by disposal event
│   │   ├── date, ticker, quantity, proceeds
│   │   └── matches: Vec<Match>        // Matches belong to disposal
│   │       ├── rule, quantity, cost, gain_loss
│   │       └── acquisition_date (for B&B)
│   ├── total_gain, total_loss, net
│   └── carried_loss (from prior years)
└── holdings: Vec<Section104Pool>      // End-state after all disposals
```

### Key Changes Summary

| Current                | Proposed                          | Rationale                                                         |
| ---------------------- | --------------------------------- | ----------------------------------------------------------------- |
| `tax_year: i32`        | `period: TaxPeriod`               | Validated type serializing to "2023/24", prevents invalid periods |
| Flat `matches` array   | `disposals[].matches[]`           | Preserves disposal context                                        |
| Single tax year        | Multi-year `tax_years` array      | Supports full transaction history                                 |
| No acquisition date    | `acquisition_date` on B&B matches | Required for audit trail                                          |
| Floating-point strings | Rounded decimal strings           | Cleaner output, correct precision                                 |

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every disposal in the output can be traced to its source sale transaction in the input
- **SC-002**: Every B&B rule match displays both disposal and acquisition dates
- **SC-003**: Tax year summaries match HMRC self-assessment categories (total gains, total losses, net)
- **SC-004**: Monetary values display with exactly 2 decimal places (no floating-point artifacts)
- **SC-005**: Users can generate a tax-year-specific report in a single operation
- **SC-006**: The data model supports generating both machine-readable (JSON) and human-readable (text/PDF) output from the same source

## Clarifications

### Session 2025-12-08

- Q: Should tax year periods use arbitrary strings or a validated type? → A: Use a dedicated `TaxPeriod` type that validates consecutive years (e.g., rejects "2023/27") and serializes to "2023/24" format

## Assumptions

- UK tax year boundaries (April 6 - April 5) are fixed and well-understood
- The tool focuses on UK CGT rules; multi-currency support is out of scope
- Section 104 pool tracking continues to use high-precision decimals internally
- Backward compatibility with existing test files is not required (tests can be migrated)
- Report formatters will be separate components consuming this data model
