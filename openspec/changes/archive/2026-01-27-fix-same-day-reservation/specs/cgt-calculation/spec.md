## MODIFIED Requirements

### Requirement: Bed and Breakfast Rule

The system SHALL match disposals with acquisitions within 30 days after sale, subject to Same Day matching priority per TCGA92/S106A(9).

When evaluating a potential B&B acquisition date, the system SHALL reserve shares needed for Same Day matching on that date before allowing earlier disposals to consume them. The available quantity for B&B matching SHALL be: `acquisition_quantity - min(same_day_disposal_quantity, acquisition_quantity)`.

#### Scenario: B&B match

- **WHEN** shares are repurchased within 30 days after sale
- **THEN** match chronologically (earliest purchase first)
- **AND** skip if repurchase is beyond 30 days

#### Scenario: Same Day reservation priority

- **WHEN** disposal D1 on Day 1 could B&B match to acquisition A2 on Day 2
- **AND** disposal D2 on Day 2 could Same Day match to acquisition A2
- **AND** A2 quantity is less than D1 + D2 combined
- **THEN** D2's Same Day claim SHALL be satisfied first
- **AND** D1's B&B match SHALL only use remaining shares after Same Day reservation

#### Scenario: Same Day reservation with sufficient shares (data-driven)

- **WHEN** `tests/inputs/SameDayReservation.cgt` is reported to JSON
- **THEN** `tests/json/SameDayReservation.json` shows Same Day matching fully satisfied before B&B from earlier disposals
- **AND** earlier disposals' B&B matches use only shares remaining after Same Day reservation

#### Scenario: Same Day disposal exceeds acquisition

- **WHEN** same-day disposals on an acquisition date exceed the acquisition quantity
- **THEN** the reservation SHALL be capped at the acquisition quantity
- **AND** excess same-day disposal shares proceed to B&B or S104 matching per normal rules
