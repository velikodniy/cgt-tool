## Context

The current `Matcher` implementation relies on O(N^2) simulation to determine holdings for corporate actions, leading to logic duplication between `matcher` and `acquisition_ledger`. We need to refactor this to a single-pass O(N) architecture while preserving correctness.

## Goals / Non-Goals

**Goals:**

- **Performance**: O(N) complexity for matching.
- **Maintainability**: Single source of truth for matching logic.
- **Correctness**: Preserved behavior for complex scenarios (B&B with intervening splits).
- **Cleanup**: More idiomatic Rust code in models.

**Non-Goals:**

- Changing public API surfaces outside of `cgt-core`.
- Changing tax rules or calculations.

## Decisions

### 1. Single-Pass Chronological Processing

**Decision**: Process `GbpTransaction`s strictly in order.
**Rationale**: By applying corporate actions to the live ledger as they are encountered, we eliminate the need to simulate historical states. The ledger state is always "current".

### 2. Peek-Forward Bed & Breakfast

**Decision**: When processing a Sell, scan forward in the sorted transaction list (limited to 30 days) to identify B&B matches.
**Rationale**: Avoids a multi-pass approach. The lookahead is constant-time (relative to total history) due to the fixed day window.
**Detail**: The scanner must track `cumulative_split_ratio` as it iterates forward to correctly adjust quantities for intervening corporate actions.

### 3. Future Consumption Tracking

**Decision**: Use a `future_consumption: HashMap<usize, Decimal>` map.

- **Key**: Index of the Buy transaction in the processed list.
- **Value**: Quantity of shares already consumed by a prior B&B Sell.
  **Rationale**: Stores the result of the "Peek-Forward" so that when the Buy is eventually processed, we know some of its shares are already gone.

### 4. Deprecation of Simulation

**Decision**: Delete `AcquisitionLedger::calculate_remaining_at_event` and `Matcher::build_ledgers`.
**Rationale**: Obsolete in the new architecture.

### 5. Idiomatic Refactoring

**Decision**:

- Use `serde(rename_all)` for `Operation` deserialization.
- Extract `TaxPeriod` year limits to constants.
  **Rationale**: Improves code quality and reduces boilerplate.

## Risks / Trade-offs

- [Risk] Lookahead logic missing intervening events (Splits) -> Mitigation: Port the split-ratio logic from the current B&B matcher into the peek loop.
- [Risk] Index misalignment for `future_consumption` -> Mitigation: The `preprocess` step (sort/merge) must run once, producing a stable list for all indexing.

## Migration Plan

1. Refactor `models.rs` (isolated cleanup).
2. Implement new matcher loop with `future_consumption`.
3. Port `same_day`, `bed_and_breakfast`, and `section104` logic.
4. Wire the new matcher into `calculator.rs`.
5. Run full test suite and compare golden outputs.
6. Remove obsolete simulation code.

## Open Questions

None.
