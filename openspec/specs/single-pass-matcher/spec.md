# Single-Pass Matcher Specification

## Purpose

Ensure the matcher processes transactions in O(N) time using a single chronological pass with peek-forward B&B matching and future consumption tracking.

## Requirements

### Requirement: Single-pass matching preserves rule order

The matcher MUST process transactions in chronological order and apply matching in this order: Same Day, Bed and Breakfast (30 days), then Section 104 pool.

#### Scenario: Basic ordering and determinism

- **WHEN** input file `tests/inputs/single_pass_ordering.cgt` is processed
- **THEN** output MUST match `tests/json/single_pass_ordering.json`

### Requirement: Bed and Breakfast lookahead accounts for splits

The matcher MUST apply split and unsplit ratios that occur between the disposal date and the acquisition date when calculating B&B matched quantities.

#### Scenario: B&B with intervening split

- **WHEN** input file `tests/inputs/single_pass_bnb_split.cgt` is processed
- **THEN** output MUST match `tests/json/single_pass_bnb_split.json`

### Requirement: Future consumption prevents double-counting

If a future Buy is claimed by a prior B&B Sell, the matcher MUST reduce the available quantity for that Buy so it is not added to the Section 104 pool or matched again.

Same-day reservation for that future acquisition date MUST be tracked across all same-day lots for the same ticker, including when those lots are non-adjacent in the transaction stream.

#### Scenario: Future buy consumed by earlier sell

- **WHEN** input file `tests/inputs/single_pass_future_consumption.cgt` is processed
- **THEN** output MUST match `tests/json/single_pass_future_consumption.json`

#### Scenario: Interleaved same-day buys

- **WHEN** same-day buys for a ticker are separated by other tickers' transactions
- **THEN** total Same Day reservation for that date+ticker MUST not be applied per-lot independently
- **AND** B&B availability MUST reflect aggregate date-level reservation

### Requirement: Unmatched disposals are rejected

If a disposal cannot be fully matched after Same Day, B&B, and Section 104 passes, the matcher MUST return an error instead of silently producing a partial disposal.

#### Scenario: Oversell rejection

- **WHEN** input contains a SELL quantity larger than available matched shares
- **THEN** processing MUST fail with an invalid transaction error indicating unmatched quantity

### Requirement: Corporate actions use live holdings state

Dividend and capital return adjustments MUST apply to the holdings state at the time of the event, without simulating historical matching, and MUST preserve existing outputs.

#### Scenario: Corporate action before disposal

- **WHEN** input file `tests/inputs/single_pass_corp_action.cgt` is processed
- **THEN** output MUST match `tests/json/single_pass_corp_action.json`

### Requirement: Multi-currency behavior remains unchanged

The matcher MUST operate on GBP-normalized amounts and MUST NOT perform additional FX lookups beyond the conversion step prior to matching.

#### Scenario: USD transactions with FX conversion

- **WHEN** input file `tests/inputs/single_pass_fx.cgt` is processed
- **THEN** output MUST match `tests/json/single_pass_fx.json`
