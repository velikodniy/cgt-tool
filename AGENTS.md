# cgt-tool Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-08

## Active Technologies

- Rust 2024 edition (stable workspace) + rust_decimal, chrono, pest, serde; planned XML parsing via quick-xml; error handling via anyhow/thiserror; new FX conversion crate to be added to the workspace (011-multi-currency)
- Rust 2024 edition (workspace uses stable Rust) + rust_decimal, chrono, pest (DSL parsing), anyhow/thiserror (errors), typst-as-lib (formatting; PDF not in scope for this feature), cargo workspace crates `cgt-core`, `cgt-cli`, formatters (010-better-testing)
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

- 011-multi-currency: Added Rust 2024 edition (stable workspace) + rust_decimal, chrono, pest, serde; planned XML parsing via quick-xml; error handling via anyhow/thiserror; new FX conversion crate to be added to the workspace

- 010-better-testing: Added Rust 2024 edition (workspace uses stable Rust) + rust_decimal, chrono, pest (DSL parsing), anyhow/thiserror (errors), typst-as-lib (formatting; PDF not in scope for this feature), cargo workspace crates `cgt-core`, `cgt-cli`, formatters

- 009-codebase-refactoring: Added Rust 2024 edition + pest (parsing), rust_decimal, chrono, thiserror, anyhow, typst-as-lib, toml (new), tera or minijinja (new for plain text templates)

<!-- MANUAL ADDITIONS START -->

<!-- MANUAL ADDITIONS END -->
