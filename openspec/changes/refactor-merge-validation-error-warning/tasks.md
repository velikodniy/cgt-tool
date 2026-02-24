## 1. Core type refactor in cgt-core

- [ ] 1.1 Replace `ValidationError` and `ValidationWarning` with `Severity` enum and `ValidationIssue` struct in `crates/cgt-core/src/validation.rs`
- [ ] 1.2 Implement unified `Display` for `ValidationIssue` using severity prefix
- [ ] 1.3 Update `ValidationResult` fields to use `Vec<ValidationIssue>`
- [ ] 1.4 Update all `ValidationError { .. }` and `ValidationWarning { .. }` construction sites in `validate()` to use `ValidationIssue`

## 2. Update public API and consumers

- [ ] 2.1 Update re-exports in `crates/cgt-core/src/lib.rs`
- [ ] 2.2 Update `crates/cgt-wasm/src/lib.rs` JSON bridge types and `From` impls

## 3. Verify

- [ ] 3.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
