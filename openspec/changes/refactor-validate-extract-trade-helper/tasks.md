## 1. Extract helper function

- [x] 1.1 Add `check_trade_fields()` private function to `crates/cgt-core/src/validation.rs` that checks zero quantity, negative quantity, negative price/total_value, and negative fees
- [x] 1.2 Replace duplicated checks in the Buy arm with a call to `check_trade_fields()`
- [x] 1.3 Replace duplicated checks in the Sell arm with a call to `check_trade_fields()`
- [x] 1.4 Replace duplicated checks in the CapReturn arm with a call to `check_trade_fields()`

## 2. Verify

- [x] 2.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
