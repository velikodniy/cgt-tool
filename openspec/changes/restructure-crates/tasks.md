# Tasks: Restructure Crates for Clear Responsibilities

- [x] Task 1: Rename cgt-fx to cgt-money
- [x] Task 2: Move CurrencyAmount to cgt-money
- [x] Task 3: Create cgt-format crate
- [x] Task 4: Remove formatting.rs from cgt-core
- [x] Task 5: Update cgt-formatter-plain to use cgt-format
- [x] Task 6: Update cgt-formatter-pdf to use cgt-format
- [x] Task 7: Update cgt-cli imports
- [x] Task 8: Final cleanup and verification

---

## Task 1: Rename cgt-fx to cgt-money

**Files:**

- Rename `crates/cgt-fx/` → `crates/cgt-money/`
- Update `crates/cgt-money/Cargo.toml`: change package name to `cgt-money`
- Update workspace `Cargo.toml`: rename member

**Dependencies to update:**

- `crates/cgt-core/Cargo.toml`: `cgt-fx` → `cgt-money`
- Any other crates depending on `cgt-fx`

**Verification:**

- `cargo build` succeeds
- `cargo test -p cgt-money` passes

---

## Task 2: Move CurrencyAmount to cgt-money

**Source:** `crates/cgt-core/src/models.rs` (lines 9-207)

**Target:** `crates/cgt-money/src/amount.rs` (new file)

**Changes:**

1. Create `crates/cgt-money/src/amount.rs` with `CurrencyAmount` struct and impls
2. Update `crates/cgt-money/src/lib.rs`: add `mod amount; pub use amount::CurrencyAmount;`
3. Remove `CurrencyAmount` from `crates/cgt-core/src/models.rs`
4. Update `crates/cgt-core/src/lib.rs`: re-export from `cgt_money`
5. Update imports in all files that use `CurrencyAmount`

**Note:** `CurrencyAmount` depends on:

- `iso_currency::Currency` (already in cgt-money)
- `rust_decimal::Decimal`
- `serde`, `schemars`

**Verification:**

- `cargo build` succeeds
- All tests pass

---

## Task 3: Create cgt-format crate

**New files:**

- `crates/cgt-format/Cargo.toml`
- `crates/cgt-format/src/lib.rs`

**Cargo.toml:**

```toml
[package]
name = "cgt-format"
version = "0.1.0"
edition = "2024"

[dependencies]
cgt-money = { path = "../cgt-money" }
chrono = { version = "0.4", default-features = false }
rust_decimal = "1.37"
```

**lib.rs content:**

- Move all content from `crates/cgt-core/src/formatting.rs`
- Add `CurrencyFormatter` struct with `format_amount()` and `format_unit()` methods

**Verification:**

- `cargo build -p cgt-format` succeeds
- Unit tests pass

---

## Task 4: Remove formatting.rs from cgt-core

**Changes:**

1. Delete `crates/cgt-core/src/formatting.rs`
2. Update `crates/cgt-core/src/lib.rs`: remove `pub mod formatting;`
3. Add `cgt-format` as dev-dependency if tests need it, or move tests to cgt-format

**Verification:**

- `cargo build -p cgt-core` succeeds
- All cgt-core tests pass

---

## Task 5: Update cgt-formatter-plain to use cgt-format

**Changes:**

1. Update `crates/cgt-formatter-plain/Cargo.toml`: add `cgt-format` dependency
2. Update imports: `cgt_core::formatting::*` → `cgt_format::*`
3. Remove ad-hoc `format_unit_amount` helper function
4. Use `CurrencyFormatter` for all currency formatting

**Verification:**

- `cargo test -p cgt-formatter-plain` passes
- Plain text output unchanged (regenerate fixtures if needed)

---

## Task 6: Update cgt-formatter-pdf to use cgt-format

**Changes:**

1. Update `crates/cgt-formatter-pdf/Cargo.toml`: add `cgt-format` dependency
2. Update imports: `cgt_core::formatting::*` → `cgt_format::*`
3. Use `CurrencyFormatter` for all currency formatting

**Verification:**

- `cargo test -p cgt-formatter-pdf` passes
- PDF output unchanged

---

## Task 7: Update cgt-cli imports

**Changes:**

1. Update any direct `cgt_core::formatting` imports to `cgt_format`
2. Update any direct `CurrencyAmount` imports to come from `cgt_money` (or via re-export)

**Verification:**

- `cargo build -p cgt-cli` succeeds
- CLI integration tests pass

---

## Task 8: Final cleanup and verification

**Actions:**

1. Run `cargo fmt`
2. Run `cargo clippy` - ensure no warnings
3. Run `cargo test` - all tests pass
4. Verify dependency graph matches design
5. Update any remaining documentation

**Verification:**

- `cargo build --release` succeeds
- `cargo test` all pass
- `cargo clippy` clean

---

## Order of Execution

1. Task 1 (rename cgt-fx)
2. Task 2 (move CurrencyAmount)
3. Task 3 (create cgt-format)
4. Task 4 (remove formatting from cgt-core)
5. Task 5 (update plain formatter)
6. Task 6 (update PDF formatter)
7. Task 7 (update CLI)
8. Task 8 (final verification)

Tasks 5-7 can be done in parallel after Task 4 completes.
