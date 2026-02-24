## ADDED Requirements

### Requirement: All-rules fixture exercises all three matching rules with valid holdings

A test fixture MUST exist that exercises Same Day, Bed & Breakfast, and Section 104 matching in a single input, with each disposal backed by a sufficient holding at the time of the sell.

#### Scenario: All-rules fixture validity

- **WHEN** the all-rules fixture is processed
- **THEN** it MUST include at least one Same Day match, one B&B match, and one S104 match
- **AND** every disposal MUST have a holding >= disposal quantity at the time of the sell
- **AND** output MUST match the corresponding golden files

### Requirement: Converter output is sorted for external calculator compatibility

The cross-validation converter MUST sort its output by date, with acquisitions before disposals on the same day, so that external calculators that process transactions sequentially can handle same-day ordering.

#### Scenario: Same-day disposal-before-acquisition in source

- **WHEN** input contains a disposal before an acquisition on the same day
- **AND** the input is converted for an external calculator
- **THEN** the converted output MUST order the acquisition before the disposal for that day

#### Scenario: Previously failing cross-validation passes

- **WHEN** inputs with same-day sell-before-buy ordering are converted and validated against external calculators
- **THEN** results MUST match for all tax years
