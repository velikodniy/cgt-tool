# CGT Tool - Agent Instructions

Capital Gains Tax calculator for UK assets implementing HMRC share matching rules.

## Principles

### Deep Modules

Provide powerful functionality through simple interfaces. Hide implementation details. If a module exposes its internal complexity, it is a design failure.

### Safety First

- No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production code
- No `#[allow(...)]` attributes to suppress warnings - fix the underlying issue instead
- Explicit error handling with `thiserror` (libraries) and `anyhow` (CLI)
- Prefer immutable data and strict typing

### Tests Are Sacred

- **Never remove tests** without proving they are incorrect
- **Never modify tests** to make code pass
- A feature is incomplete until fully tested

### Domain Mastery

Verify implementations against HMRC guidance (`TAX_RULES.md`). Do not guess tax calculations.

## Commands

```bash
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # All tests
cargo clippy               # Lint (strict: denies unwrap/expect/panic)
cargo fmt                  # Format
```

## Structure

```
crates/
├── cgt-core/              # Parsing, calculation, data model
├── cgt-cli/               # CLI binary
├── cgt-mcp/               # MCP server for AI assistants
├── cgt-money/             # Currency and FX conversion
├── cgt-formatter-plain/   # Plain text output
├── cgt-formatter-pdf/     # PDF output (Typst)
├── cgt-format/            # Output format trait
└── cgt-converter/         # Broker CSV converters
tests/
├── inputs/                # .cgt test files
├── json/                  # Expected JSON
└── plain/                 # Expected plain text
```

## Rules

- Rust 2024 edition, `rust_decimal` for money, `chrono` for dates
- `pest` grammar for DSL parsing (`cgt-core/src/parser.pest`)
- Unix newlines, standard Rust naming

## DSL Syntax Changes

When modifying DSL grammar or transaction formats:

- Update `crates/cgt-core/src/parser.pest` (grammar)
- Update `crates/cgt-core/src/parser.rs` (parsing logic)
- Update MCP tool descriptions in `crates/cgt-mcp/src/server.rs`
- Update DSL resources in `crates/cgt-mcp/src/resources.rs`
- Update `README.md` syntax documentation

## Never

- Remove or modify tests without proving incorrectness
- Commit code that fails `cargo clippy` or `cargo test`
- Guess tax calculations—verify against `TAX_RULES.md`

## Commits

Format: `type: description` (feat/fix/test/docs/chore/refactor)

Run `cargo fmt && cargo clippy` before committing.

## Domain

- **Matching order**: Same Day → Bed & Breakfast (30 days) → Section 104 Pool
- **Tax year**: 6 April to 5 April (e.g., 2024/25 = 6 Apr 2024 – 5 Apr 2025)
- **Reference**: `TAX_RULES.md`, HMRC CG51500-CG51600

## OpenSpec

Spec-driven development. See `openspec/AGENTS.md` for workflow.

- `openspec/specs/` — Current truth (what IS built)
- `openspec/changes/` — Proposals (what SHOULD change)
- `openspec/project.md` — Project context
