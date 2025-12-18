# Change: Integrate Matcher Module and Remove Duplicate Calculator Logic

## Why

The `crates/cgt-core/src/matcher/` module provides a well-modularized O(n) matching implementation with separate files for each rule (same_day.rs, bed_and_breakfast.rs, section104.rs, acquisition_ledger.rs). However, it was never integrated - all logic remains duplicated in the monolithic 629-line `calculator.rs`. This causes:

1. Maintenance burden from duplicate implementations
2. Confusion about which code is authoritative
3. Wasted effort on unused, better-structured code

## What Changes

- Integrate `Matcher` from the `matcher/` module into `calculator.rs`
- Remove duplicate matching logic from `calculator.rs` (AcquisitionTracker, InternalMatch, same-day/B&B/S104 passes)
- Keep tax year filtering, disposal grouping, and FX conversion in `calculator.rs`
- Adapt `Matcher` to work with `GbpTransaction` (post-FX-conversion)

## Impact

- Affected specs: None (same behavior, different internal structure)
- Affected code:
  - `crates/cgt-core/src/calculator.rs` - Refactored to use Matcher
  - `crates/cgt-core/src/matcher/` - Minor adaptations for GbpTransaction
- Risk: Low - comprehensive test coverage via golden file tests will catch any regressions
