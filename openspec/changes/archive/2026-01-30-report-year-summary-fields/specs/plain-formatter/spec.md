## MODIFIED Requirements

### Requirement: Summary Table

The system SHALL show tax year, disposal count, net gain, total gains (before losses), total losses, gross proceeds (for SA108 Box 21), exemption, and taxable gain per year.

#### Scenario: Multi-year summary

- **WHEN** disposals span multiple tax years
- **THEN** show one row per tax year with gross proceeds

#### Scenario: Gross proceeds for SA108

- **WHEN** generating summary table
- **THEN** the Proceeds column displays gross proceeds (before sell fees)
- **AND** this value corresponds to SA108 Box 21 "Disposal proceeds"

#### Scenario: Summary totals and counts

- **WHEN** generating the summary table for any tax year
- **THEN** the disposal count equals the number of grouped disposals in that year after same-day aggregation (CG51560)
- **AND** the gains column equals the tax year's `total_gain`
- **AND** the losses column equals the tax year's `total_loss`
