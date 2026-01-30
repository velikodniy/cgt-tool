## ADDED Requirements

### Requirement: Tax Year Disposal Count

The system SHALL include a `disposal_count` in each `TaxYearSummary`, representing the number of grouped disposals per tax year after applying same-day aggregation (CG51560).

#### Scenario: Data-driven disposal count (multi-year)

- **WHEN** `tests/inputs/RealisticMultiYear.cgt` is reported to JSON
- **THEN** `tests/json/RealisticMultiYear.json` includes `disposal_count` for each tax year matching the number of grouped disposals in that year

#### Scenario: Data-driven disposal count (multi-currency)

- **WHEN** `tests/inputs/MultiCurrencySameDay.cgt` is reported to JSON
- **THEN** `tests/json/MultiCurrencySameDay.json` includes `disposal_count` values that match the number of grouped disposals per tax year regardless of FX conversion

#### Scenario: Same-day sells are grouped

- **WHEN** `tests/inputs/SameDayMerge.cgt` is reported to JSON
- **THEN** `tests/json/SameDayMerge.json` includes `disposal_count` equal to the number of grouped disposals after same-day aggregation

#### Scenario: No disposals for filtered year

- **WHEN** `calculate()` is called with a `tax_year_start` that has no disposals
- **THEN** the resulting `TaxYearSummary` includes `disposal_count` of `0`
