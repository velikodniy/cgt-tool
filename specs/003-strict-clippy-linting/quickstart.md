# Quickstart: Strict Clippy Linting

**Feature**: 003-strict-clippy-linting
**Date**: 2025-12-08

## Overview

This feature enforces strict Clippy linting by:

1. Creating `clippy.toml` with deny-level lints for panicking code
2. Removing all `#[allow(...)]` attributes
3. Replacing all `.unwrap()` calls with proper error handling

## Implementation Order

### Step 1: Create clippy.toml

Create `clippy.toml` at workspace root:

```toml
# Workspace-level Clippy configuration
# Strict linting - no panicking code in production

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
```

**Verification**: `cargo clippy` will now report all violations

### Step 2: Add New Error Variants

Add to `crates/cgt-core/src/error.rs`:

```rust
#[error("Unexpected parser state: expected {expected}")]
UnexpectedParserState {
    expected: &'static str,
},

#[error("Invalid date: year {year} is out of valid range")]
InvalidDateYear {
    year: i32,
},
```

### Step 3: Fix parser.rs Unwraps

Replace each `.unwrap()` with `.ok_or_else()` pattern:

```rust
// Pattern for pest iterator .next()
let token = inner.next()
    .ok_or_else(|| CgtError::UnexpectedParserState {
        expected: "token description"
    })?;
```

Files affected:

- `crates/cgt-core/src/parser.rs` (~18 replacements)

### Step 4: Fix calculator.rs

1. Remove `#[allow(clippy::needless_range_loop)]`
2. Refactor loop to use `.iter().enumerate()` or `.iter_mut().enumerate()`
3. Replace date creation unwraps:

```rust
let start_date = NaiveDate::from_ymd_opt(tax_year_start, 4, 6)
    .ok_or_else(|| CgtError::InvalidDateYear { year: tax_year_start })?;
```

### Step 5: Fix Test Code

Add lint exemption and replace `.unwrap()` with `.expect()`:

```rust
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    // Use .expect() with descriptive messages
    let date = NaiveDate::from_ymd_opt(2023, 1, 1)
        .expect("2023-01-01 is a valid date");
}
```

Files affected:

- `crates/cgt-core/tests/parser_tests.rs` - Remove `#[allow(unused_imports)]`, use expect()
- `crates/cgt-core/tests/matching_tests.rs` - Use expect()

## Verification Commands

```bash
# Check linting passes (production code)
cargo clippy --lib --bins -- -D warnings

# Check tests compile (with exemptions)
cargo clippy --tests -- -D warnings

# Run all tests
cargo test

# Verify no unwrap in production code
grep -r "\.unwrap()" crates/*/src/ && echo "FAIL: unwrap found" || echo "PASS"

# Verify no allow attributes in production code
grep -r "#\[allow(" crates/*/src/ && echo "FAIL: allow found" || echo "PASS"
```

## Common Patterns

### Option to Result

```rust
// Before
let x = option.unwrap();

// After
let x = option.ok_or_else(|| CgtError::SomeVariant { ... })?;
```

### Test Assertions

```rust
// Before
let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

// After
let date = NaiveDate::from_ymd_opt(2023, 1, 1)
    .expect("2023-01-01 is a valid date");
```

### Iterator Refactoring

```rust
// Before (with allow)
#[allow(clippy::needless_range_loop)]
for i in 0..vec.len() {
    do_something(vec[i], i);
}

// After (idiomatic)
for (i, item) in vec.iter().enumerate() {
    do_something(item, i);
}
```

## Success Criteria Checklist

- [ ] `clippy.toml` created at workspace root
- [ ] `cargo clippy --lib --bins` passes with zero warnings
- [ ] No `#[allow(...)]` in `src/` directories
- [ ] No `.unwrap()` in `src/` directories
- [ ] All tests pass (`cargo test`)
- [ ] Test code uses `.expect()` with descriptive messages
