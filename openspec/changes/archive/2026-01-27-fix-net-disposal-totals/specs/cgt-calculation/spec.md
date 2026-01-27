## ADDED Requirements

### Requirement: Tax year total gain/loss aggregation

The system SHALL compute tax-year `total_gain` and `total_loss` from the net result of each disposal after applying CG51560 matching rules and allowable expenditure per TCGA92/S38 (CG15150, CG15250). A disposal SHALL contribute either to `total_gain` (net >= 0) or to `total_loss` (net < 0); `net_gain` SHALL equal `total_gain - total_loss`.

#### Scenario: Mixed rule disposal netting (data-driven)

- **WHEN** `tests/inputs/NetDisposalTotalsMixed.cgt` is reported to JSON
- **THEN** `tests/json/NetDisposalTotalsMixed.json` reports `total_gain` and `total_loss` based on net per-disposal results
- **AND** a disposal with both positive and negative match legs contributes only its net result to totals

#### Scenario: Net zero disposal does not inflate totals

- **WHEN** a disposal has match legs that sum to zero
- **THEN** it contributes to neither `total_gain` nor `total_loss`

#### Scenario: Multi-currency totals use converted GBP

- **WHEN** a disposal includes non-GBP transactions that are converted to GBP during matching
- **THEN** `total_gain` and `total_loss` are aggregated from the GBP proceeds and allowable costs used in disposal matches
