## 1. Restructure MatchResult

- [x] 1.1 Update `MatchResult` in `matcher/mod.rs` to contain a `match_detail: Match` field and remove the duplicated fields (`rule`, `quantity`, `allowable_cost`, `gain_or_loss`, `acquisition_date`)
- [x] 1.2 Update `MatchResult` constructor in `matcher/same_day.rs` to build an inner `Match`
- [x] 1.3 Update `MatchResult` constructor in `matcher/bed_and_breakfast.rs` to build an inner `Match`
- [x] 1.4 Update `MatchResult` constructor in `matcher/section104.rs` to build an inner `Match`

## 2. Update consumers

- [x] 2.1 Simplify `group_matches_into_disposals` in `calculator.rs` to extract `match_detail` directly
- [x] 2.2 Update all other `MatchResult` field accesses in `calculator.rs` (e.g., `m.disposal_date`, `m.quantity` → `m.match_detail.quantity`)

## 3. Verify

- [x] 3.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
