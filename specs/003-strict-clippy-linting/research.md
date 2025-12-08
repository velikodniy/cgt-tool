# Research: Strict Clippy Linting

**Feature**: 003-strict-clippy-linting
**Date**: 2025-12-08

## Research Topics

### 1. Clippy.toml Configuration for Strict Linting

**Decision**: Create `clippy.toml` at workspace root with deny-level lints

**Rationale**:

- Centralized configuration applies to all crates in workspace
- `clippy.toml` is the modern, idiomatic approach for Clippy configuration
- Separates lint configuration from source code
- Easy to update without modifying source files

**Configuration**:

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

**Note**: Test code exemptions are handled via `#[cfg_attr(test, allow(...))]` or by running clippy with `--tests` flag separately.

**Alternatives considered**:

- `#![deny(...)]` in each crate - Clutters source files, harder to maintain
- `Cargo.toml` `[lints]` section - Requires Rust 1.74+, less flexible for clippy-specific config
- `.cargo/config.toml` with RUSTFLAGS - Global, affects all builds including dependencies

### 2. Error Handling Library

**Decision**: Keep `thiserror` (already in use)

**Rationale**:

- Zero-cost abstractions - no runtime overhead
- Derives `std::error::Error` and `Display` automatically
- Already integrated in cgt-core
- Stable, widely adopted in Rust ecosystem
- Works well with `anyhow` in CLI layer

**Alternatives considered**:

- `anyhow` for library - Loses type information, not suitable for libraries
- `miette` - Rich diagnostics but heavier, overkill for this scope
- `error-stack` - Modern but adds complexity without clear benefit here

### 3. Replacing unwrap() in Parser Code

**Decision**: Use `ok_or_else()` with contextual `CgtError` variants

**Rationale**:

- Parser iterators from pest return `Option<Pair>` for `.next()`
- Grammar guarantees tokens exist, but we need graceful fallback
- `ok_or_else()` lazily constructs error only on failure
- Error messages include what token was expected

**Pattern**:

```rust
// Before
let date_pair = inner.next().unwrap();

// After
let date_pair = inner.next()
    .ok_or_else(|| CgtError::UnexpectedParserState {
        expected: "date token"
    })?;
```

**Alternatives considered**:

- `.expect()` - Still panics, just with message
- Match expressions - Verbose for simple extractions
- Custom iterator wrapper - Over-engineering

### 4. Replacing unwrap() in Date Creation

**Decision**: Return `CgtError::InvalidDateYear` for invalid date construction

**Rationale**:

- `NaiveDate::from_ymd_opt()` returns `Option<NaiveDate>`
- Invalid years (e.g., year 0, overflow) return None
- Tax year dates should be valid for reasonable years but need fallback
- Error includes the problematic year value

**Pattern**:

```rust
// Before
let start_date = NaiveDate::from_ymd_opt(tax_year_start, 4, 6).unwrap();

// After
let start_date = NaiveDate::from_ymd_opt(tax_year_start, 4, 6)
    .ok_or_else(|| CgtError::InvalidDateYear { year: tax_year_start })?;
```

**Alternatives considered**:

- Validate year range upfront only - Good addition but doesn't replace error handling
- Use chrono's panicking `NaiveDate::from_ymd()` - Deprecated, wrong approach

### 5. Handling Existing allow Attributes

**Decision**: Remove allows by fixing underlying code issues

**Current allows**:

1. `#[allow(unused_imports)]` in `parser_tests.rs:1` - Remove unused import or use it
2. `#[allow(clippy::needless_range_loop)]` in `calculator.rs:27` - Refactor to idiomatic iterator

**Pattern for needless_range_loop**:

```rust
// Before (with allow)
#[allow(clippy::needless_range_loop)]
for i in 0..vec.len() {
    // uses vec[i] and i
}

// After (idiomatic)
for (i, item) in vec.iter().enumerate() {
    // uses item and i
}
```

**Alternatives considered**:

- Keep allows with justification comments - Defeats strict linting goal
- Disable lint globally - Wrong direction

### 6. Test Code Strategy

**Decision**: Use `.expect("descriptive message")` in test code, exempt from `expect_used`

**Rationale**:

- Test assertions with `.expect()` provide clear failure context
- Panicking in tests is acceptable - it fails the test with a message
- Full Result handling in tests adds noise without benefit
- Tests can be exempted from production-only lints

**Implementation options**:

1. `#![cfg_attr(test, allow(clippy::expect_used))]` at crate level
2. Run `cargo clippy` separately for `--lib` and `--tests`
3. Use `#[allow(clippy::expect_used)]` on test modules only

**Pattern**:

```rust
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    // Tests can use .expect() freely
    let date = NaiveDate::from_ymd_opt(2023, 1, 1)
        .expect("test date should be valid");
}
```

### 7. New CgtError Variants

**Decision**: Add two new variants for unwrap replacement

**New variants**:

```rust
#[derive(Error, Debug)]
pub enum CgtError {
    // Existing variants unchanged...
    #[error("Unexpected parser state: expected {expected}")]
    UnexpectedParserState { expected: &'static str },

    #[error("Invalid date: year {year} is out of valid range")]
    InvalidDateYear { year: i32 },
}
```

**Rationale**:

- Specific variants enable meaningful error messages
- `&'static str` for expected - compile-time strings, no allocation
- Structured errors allow programmatic handling if needed

**Alternatives considered**:

- Reuse `InvalidTransaction` for everything - Less informative
- String-only errors - Lose type safety

## Summary

All research topics resolved. Key decisions:

- Use `clippy.toml` at workspace root for centralized configuration
- Keep `thiserror` for error handling
- Add `UnexpectedParserState` and `InvalidDateYear` error variants
- Use `ok_or_else()` pattern for Option-to-Result conversion
- Allow `.expect()` in test code only

Ready for Phase 1 design.
