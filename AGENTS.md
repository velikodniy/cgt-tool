# cgt-tool Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-08

## Active Technologies

- Rust 2024 edition + pest (parsing), thiserror (error types), anyhow (CLI error handling), chrono (dates), rust_decimal (numbers), typst-as-lib (v0.15+), typst-pdf, typst-assets (for embedded fonts)

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

- 009-codebase-refactoring: Added Rust 2024 edition + pest (parsing), rust_decimal, chrono, thiserror, anyhow, typst-as-lib, toml (new), tera or minijinja (new for plain text templates)
- 008-pdf-typst-formatter: Added Rust 2024 edition (matching existing crates) + typst-as-lib (v0.15+), typst-pdf, typst-assets (for embedded fonts)
- 007-plain-formatter: Added Rust 2024 edition (existing) + cgt-core (existing), clap (CLI), rust_decimal, chrono
- 006-multi-ticker: No new dependencies (multi-ticker Section 104 pooling with ticker normalization)
- 005-internal-data-model: Added Rust 2024 edition + serde, serde_json, chrono, rust_decimal, thiserror, schemars (JsonSchema)
- 004-test-validation: No new dependencies (documentation and test verification only)
- 003-strict-clippy-linting: Enforced strict Clippy linting, replaced all unwraps with proper error handling
- 002-dsl-enhancements: Added TAX, EXPENSES, RATIO keywords for improved DSL readability
- 001-cgt-cli: Initial CGT CLI tool with PEG parser and UK tax rules (Same Day, B&B, Section 104)

<!-- MANUAL ADDITIONS START -->

<!-- MANUAL ADDITIONS END -->
