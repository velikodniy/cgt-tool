## 1. Core aggregation update

- [x] 1.1 Update `cgt-core` tax-year aggregation (`calculate_totals`) to sum net disposal results into `total_gain` / `total_loss`.
- [x] 1.2 Add unit tests for mixed-rule disposals (Same Day + B&B, Same Day + S104) and net-zero disposal totals.

## 2. Reporting and docs

- [x] 2.1 Update `docs/tax-rules.md` to document net-per-disposal totals with HMRC references (CG51560, CG15150, CG15250).
- [x] 2.2 Update plain report summary note to clarify gains/losses are net per disposal and align with SA108 guidance.

## 3. Verification

- [x] 3.1 Run `cargo fmt && cargo clippy && cargo test`; review any golden output changes and update only after manual validation of HMRC-aligned totals.
