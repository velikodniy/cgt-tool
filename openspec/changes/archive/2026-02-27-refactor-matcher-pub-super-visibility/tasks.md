## 1. Restrict visibility

- [x] 1.1 Change `pub fn get_ledger_mut` to `pub(super) fn get_ledger_mut` in `crates/cgt-core/src/matcher/mod.rs`
- [x] 1.2 Change `pub fn get_pool_mut` to `pub(super) fn get_pool_mut` in `crates/cgt-core/src/matcher/mod.rs`

## 2. Verify

- [x] 2.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
