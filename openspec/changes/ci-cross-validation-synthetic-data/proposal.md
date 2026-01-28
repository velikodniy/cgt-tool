## Why

The cross-validation script (`scripts/cross-validate.py`) exists and works locally but is not part of CI, meaning calculation discrepancies against external UK CGT calculators (cgt-calc, cgtcalc) could go undetected. Additionally, the existing test fixtures lack comprehensive RSU vesting patterns with Schwab JSON format, limiting converter validation coverage.

## What Changes

- Add a separate GitHub Actions workflow (`cross-validate.yml`) that runs cross-validation against external calculators on manual trigger only
- Create a comprehensive synthetic test fixture (`SyntheticComplex.cgt`) covering all matching rules and edge cases over 5 tax years
- Create matching Schwab-format JSON files (`tests/schwab/synthetic-awards.json`, `tests/schwab/synthetic-transactions.json`) for converter testing
- Generate golden files (JSON and plain text) for the new fixture

## Capabilities

### New Capabilities

None - this change extends existing capabilities without introducing new ones.

### Modified Capabilities

- `ci-cd`: Add cross-validation workflow that runs external calculator comparisons on manual trigger
- `testing`: Add comprehensive synthetic fixture with RSU vesting patterns and Schwab JSON format test data

## Impact

- `.github/workflows/cross-validate.yml`: New workflow file (manual trigger only)
- `tests/inputs/SyntheticComplex.cgt`: New comprehensive test fixture
- `tests/schwab/`: New directory with `synthetic-awards.json` and `synthetic-transactions.json`
- `tests/json/SyntheticComplex.json`: Golden file for JSON output
- `tests/plain/SyntheticComplex.txt`: Golden file for plain text output
- CI runners: Uses macos-latest for both calculators in a single job

## Verification

- Cross-validation workflow will validate cgt-tool calculations against:
  - KapJI/capital-gains-calculator (Python, via `uvx cgt-calc`)
  - mattjgalloway/cgtcalc (Swift, built from source)
- Discrepancies greater than £1 per tax year will be reported
- Synthetic fixture will be verified against both external calculators
- Converter round-trip: Schwab JSON → CGT DSL → report should match expected output
