## 1. Remove dead field

- [ ] 1.1 Remove `remaining_amount` field from `AcquisitionLot` struct
- [ ] 1.2 Remove `remaining_amount: amount` assignment from `AcquisitionLot::new()`
- [ ] 1.3 Replace `self.remaining_amount` with `self.original_amount` in `available()`
- [ ] 1.4 Replace `self.remaining_amount` with `self.original_amount` in `held_for_adjustment()`

## 2. Verify

- [ ] 2.1 Run `cargo fmt && cargo clippy && cargo test` — all must pass with no modifications to tests
