## 1. Refactor Operation conversion

- [ ] 1.1 Add `to_gbp` method on `Operation<CurrencyAmount>` in `crates/cgt-core/src/models.rs` that moves the match arms from `Transaction::to_gbp`
- [ ] 1.2 Simplify `Transaction::to_gbp` to delegate to `self.operation.to_gbp(self.date, fx_cache)?`

## 2. Verify

- [ ] 2.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass without test modifications
