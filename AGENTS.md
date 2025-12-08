# cgt-tool Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-08

## Active Technologies

- Rust 2024 edition + pest (parsing), thiserror (error types), anyhow (CLI error handling), chrono (dates), rust_decimal (numbers)

## Project Structure

```text
crates/cgt-core/src/   # Core library
crates/cgt-cli/src/    # CLI binary
crates/cgt-core/tests/ # Integration tests
```

## Commands

```bash
cargo test              # Run all tests
cargo clippy            # Run linter (strict - denies unwrap, expect, panic)
cargo build --release   # Build release binary
```

## Code Style

- Rust 2024 edition: Follow standard conventions
- No `.unwrap()` or `.expect()` in production code - use proper error handling
- Use `thiserror` for error types, `anyhow` for CLI error handling

## Recent Changes

- 003-strict-clippy-linting: Enforced strict Clippy linting, replaced all unwraps with proper error handling
- 002-dsl-enhancements: Added TAX, EXPENSES, RATIO keywords for improved DSL readability
- 001-cgt-cli: Initial CGT CLI tool with PEG parser and UK tax rules (Same Day, B&B, Section 104)

<!-- MANUAL ADDITIONS START -->

<!-- MANUAL ADDITIONS END -->
