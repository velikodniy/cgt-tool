## MODIFIED Requirements

### Requirement: Summary Table

The system SHALL show tax year, gain, gross proceeds (for SA108 Box 21), exemption, taxable gain per year.

#### Scenario: Multi-year summary

- **WHEN** disposals span multiple tax years
- **THEN** show one row per tax year with gross proceeds

#### Scenario: Gross proceeds for SA108

- **WHEN** generating summary table
- **THEN** the Proceeds column displays gross proceeds (before sell fees)
- **AND** this value corresponds to SA108 Box 21 "Disposal proceeds"

### Requirement: Proceeds Breakdown

The plain text disposal calculation SHALL display both gross and net proceeds with clear labels, and the net figure SHALL match the gain/loss calculation.

#### Scenario: Sell with sale expenses

- **WHEN** a disposal includes sale expenses
- **THEN** the proceeds section shows gross proceeds as `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** shows net proceeds as `£<gross proceeds> - £<sale expenses> fees = £<net proceeds>`
- **AND** `<net proceeds>` equals `(unit price × quantity) - sale expenses`

#### Scenario: Sell without sale expenses

- **WHEN** sale expenses are zero
- **THEN** the proceeds section shows `<quantity> × £<unit price> = £<gross proceeds>`
- **AND** gross and net proceeds are equal
- **AND** the fees line is omitted

#### Scenario: Same-day merge with multiple sells

- **WHEN** multiple sells occur on the same day for the same ticker
- **THEN** display the weighted average unit price calculated as total_gross_proceeds / total_quantity
- **AND** the displayed price reflects the actual average, not an arbitrary individual sell price

## ADDED Requirements

### Requirement: SA108 Guidance Note

The system SHALL include a brief note in the report explaining which values correspond to SA108 boxes.

#### Scenario: Summary section note

- **WHEN** generating a report with disposals
- **THEN** include a note below the summary table indicating that Proceeds = SA108 Box 21, and that allowable costs include all fees
