# Feature Specification: PDF Typst Formatter

**Feature Branch**: `008-pdf-typst-formatter`
**Created**: 2025-12-09
**Status**: Draft
**Input**: User description: "Let's implement another formatter to produce beautiful, clear and readable PDFs using Typst. The formatter MUSTN'T require installing external tools (typst, latex, etc) it should use embeddable typst (there's a crate for this). You can use tables, lists or even diagrams if needed in the doc."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate PDF Tax Report (Priority: P1)

As a taxpayer, I want to generate a professional PDF tax report from my CGT calculations so that I can have a permanent, printable record suitable for submission to HMRC or for my personal records.

**Why this priority**: This is the core feature - without PDF generation, the feature has no value. Users need to be able to take their calculated tax data and produce a formatted document.

**Independent Test**: Can be fully tested by running the CLI with `--format pdf` option on any valid .cgt input file and verifying a valid PDF is produced that can be opened in any PDF reader.

**Acceptance Scenarios**:

1. **Given** a valid .cgt file with transactions, **When** I run the CLI with `--format pdf --year 2023`, **Then** a PDF file is generated containing the tax report
2. **Given** a .cgt file with multiple tax years of data, **When** I generate a PDF for a specific year, **Then** the PDF shows only data relevant to that tax year
3. **Given** any valid input, **When** PDF generation completes, **Then** the output is a valid PDF that opens in standard PDF readers (Adobe, Preview, Chrome)

---

### User Story 2 - View Summary Information (Priority: P1)

As a taxpayer reviewing my report, I want to see a clear summary section at the top of the PDF showing my total gains, losses, exemption used, and taxable amount so I can quickly understand my tax position.

**Why this priority**: The summary is essential for understanding the report - users need this at a glance without reading the entire document.

**Independent Test**: Generate a PDF and verify the first page contains a prominently displayed summary table with all key figures.

**Acceptance Scenarios**:

1. **Given** a generated PDF report, **When** I view the first page, **Then** I see a summary table showing tax year, total gain/loss, proceeds, exemption, and taxable gain
2. **Given** a report with losses exceeding gains, **When** I view the summary, **Then** the net position shows as a loss (negative value) with appropriate formatting

---

### User Story 3 - Review Disposal Details (Priority: P2)

As a taxpayer, I want to see detailed breakdowns of each disposal (sale) in my PDF report, including matching rules applied, so I can understand how my gains/losses were calculated and verify correctness.

**Why this priority**: After the summary, users need to drill into the details to understand and verify calculations. This supports tax compliance and record-keeping.

**Independent Test**: Generate a PDF with multiple disposals and verify each disposal is clearly presented with its matching breakdown.

**Acceptance Scenarios**:

1. **Given** a disposal matched by Same Day rule, **When** I view its details in the PDF, **Then** I see the shares matched, cost basis, proceeds, and resulting gain/loss
2. **Given** a disposal matched by Bed & Breakfast rule, **When** I view its details, **Then** I see the acquisition date of the matched shares and the calculation breakdown
3. **Given** a disposal matched by Section 104 pooling, **When** I view its details, **Then** I see the pool cost per share and the proportion used

---

### User Story 4 - View Holdings Summary (Priority: P2)

As a taxpayer, I want to see my remaining holdings (shares still owned) at the end of the tax year in my PDF report so I can track my portfolio and verify my records.

**Why this priority**: Holdings information helps users track their investment position and verify the calculator's state matches their understanding.

**Independent Test**: Generate a PDF for a scenario where shares remain after sales and verify the holdings section appears correctly.

**Acceptance Scenarios**:

1. **Given** remaining shares in my portfolio after sales, **When** I view the holdings section, **Then** I see each ticker with quantity and average cost basis
2. **Given** all shares of a ticker were sold, **When** I view the holdings section, **Then** that ticker does not appear (or shows zero)

---

### User Story 5 - View Transaction History (Priority: P3)

As a taxpayer, I want to see a list of all transactions (buys and sells) in my PDF report so I can verify the input data was processed correctly.

**Why this priority**: Transaction listing provides audit trail and helps users verify their input was parsed correctly. Lower priority as most users focus on results.

**Independent Test**: Generate a PDF and verify all input transactions appear in the transactions section.

**Acceptance Scenarios**:

1. **Given** multiple buy and sell transactions, **When** I view the transactions section, **Then** I see all transactions listed chronologically with date, type, ticker, quantity, price, and fees

---

### Edge Cases

- What happens when there are no disposals in the requested tax year? → PDF should generate with empty disposal section and zero gains
- What happens when a .cgt file has invalid data? → Error message before PDF generation, no corrupted PDF output
- What happens when output path is not writable? → Clear error message indicating the file cannot be written
- How does the system handle very long ticker symbols or large numbers? → Content should wrap or truncate gracefully without breaking layout

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate PDF output using embedded Typst (no external tool installation required)
- **FR-002**: System MUST support `--format pdf` option on the `report` command
- **FR-003**: System MUST write PDF output to a file (default: input filename with .pdf extension, or specified via `--output` flag)
- **FR-004**: PDF MUST contain a Summary section with: tax year, total gain/loss, total proceeds, annual exemption, taxable gain
- **FR-005**: PDF MUST contain a Tax Year Details section listing each disposal with matching rule breakdown
- **FR-006**: PDF MUST contain a Holdings section showing remaining positions with quantity and average cost
- **FR-007**: PDF MUST contain a Transactions section listing all buy/sell transactions chronologically
- **FR-008**: PDF MUST use professional formatting including clear headings, readable fonts, and proper spacing
- **FR-009**: PDF MUST use tables for presenting summary data and transaction lists
- **FR-010**: System MUST handle all asset event types (dividend, capital return, split, unsplit) in the transaction display
- **FR-011**: PDF MUST include a header with report title and generation date
- **FR-012**: Currency values MUST be formatted with £ symbol and appropriate decimal places
- **FR-013**: Dates MUST be formatted in UK format (DD/MM/YYYY)
- **FR-014**: System MUST return appropriate error if PDF generation fails (not silently fail)

### Key Entities

- **Tax Report**: The calculated CGT data including tax years, disposals, holdings
- **PDF Document**: The generated output file containing formatted report content
- **Typst Template**: The embedded document template defining layout and styling

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can generate a PDF report in under 5 seconds for typical input files (up to 1000 transactions)
- **SC-002**: Generated PDFs open successfully in all major PDF readers (Adobe Reader, macOS Preview, Chrome PDF viewer)
- **SC-003**: All 26 existing test cases successfully generate valid PDFs without errors
- **SC-004**: PDF content matches the plain text formatter output for all numerical values (gains, losses, costs)
- **SC-005**: PDFs are readable when printed on A4 paper without content being cut off

## Assumptions

- The `typst` crate (or similar embedded Typst library) is available and suitable for this use case
- PDF generation will be a synchronous operation completing before the CLI exits
- Default output filename will be the input filename with `.pdf` extension (e.g., `report.cgt` → `report.pdf`)
- The existing `TaxReport` data structure contains all necessary information for the PDF
- Professional formatting means: clear section headings, consistent fonts, proper alignment, adequate margins
