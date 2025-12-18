## 1. Adapt Matcher for GbpTransaction

- [x] 1.1 Update `Matcher::process()` to accept `GbpTransaction` instead of `Transaction`
- [x] 1.2 Update matcher sub-modules to work with `GbpTransaction` (Decimal prices instead of CurrencyAmount)
- [x] 1.3 Ensure `MatchResult` output matches `InternalMatch` structure

## 2. Integrate Matcher into Calculator

- [x] 2.1 Replace `AcquisitionTracker` with `AcquisitionLedger` from matcher
- [x] 2.2 Replace same-day matching pass with `same_day::match_same_day()`
- [x] 2.3 Replace B&B matching pass with `bed_and_breakfast::match_bed_and_breakfast()`
- [x] 2.4 Replace Section 104 pass with `section104::match_section_104()`
- [x] 2.5 Remove duplicate `InternalMatch` struct (use `MatchResult` from matcher)
- [x] 2.6 Keep `group_matches_into_disposals()` and tax year filtering logic

## 3. Clean Up

- [x] 3.1 Remove dead code from `calculator.rs` (old matching passes, AcquisitionTracker)
- [x] 3.2 Ensure `matcher` module is properly re-exported if needed by external crates

## 4. Validation

- [x] 4.1 Run `cargo build` to confirm no compilation errors
- [x] 4.2 Run `cargo test` to confirm all golden file tests pass
- [x] 4.3 Run `cargo clippy` to confirm no warnings
