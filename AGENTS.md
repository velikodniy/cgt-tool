# CGT Tool - Agent Instructions

Capital Gains Tax calculator for UK assets implementing HMRC share matching rules.

## Commands

```bash
cargo build                 # Debug build
cargo test                  # All tests
cargo clippy               # Lint (strict: denies unwrap/expect/panic)
cargo fmt                  # Format
cargo llvm-cov             # Coverage (requires cargo-llvm-cov)
python3 scripts/cross-validate.py tests/inputs/*.cgt  # Cross-validation
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
web/                       # WASM demo web interface
tests/
├── inputs/                # .cgt test files (fixtures)
├── json/                  # Expected JSON (golden files)
└── plain/                 # Expected plain text (golden files)
```

## Principles

- **Deep Modules**: Simple interfaces hiding implementation complexity
- **Safety First**: No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()`, no `#[allow(...)]`. Use `thiserror` (libs) and `anyhow` (CLI).
- **Tests Are Sacred**: Never remove or modify tests without proving incorrectness
- **Domain Mastery**: Verify against HMRC guidance (`docs/tax-rules.md`). Do not guess tax calculations.

## Rules

- Rust 2024 edition, `rust_decimal` for money, `chrono` for dates
- `pest` grammar for DSL parsing (`cgt-core/src/parser.pest`)
- IO-free core: calculation logic has no IO, is WASM-friendly
- Bundled FX rates: HMRC rates embedded at compile time; runtime override via `--fx-folder`
- Prefer immutable data and strict typing
- Unix newlines, standard Rust naming
- No long separator lines in comments (e.g., `// ====...` or `// ----...`)

## DSL Syntax Changes

When modifying DSL grammar or transaction formats:

- Update `crates/cgt-core/src/parser.pest` (grammar)
- Update `crates/cgt-core/src/parser.rs` (parsing logic)
- Update MCP tool descriptions in `crates/cgt-mcp/src/server.rs`
- Update DSL resources in `crates/cgt-mcp/src/resources.rs`
- Update `README.md` syntax documentation

## After Major Changes

When modifying matching rules, corporate actions, tax calculations, FX conversion, or DSL syntax:

- Update `docs/spec.md` to reflect new or changed behavior
- Add or update golden-file tests (`.cgt` → `.json`)
- Run cross-validation: `python3 scripts/cross-validate.py tests/inputs/*.cgt`

## Domain

- **Matching order**: Same Day → Bed & Breakfast (30 days) → Section 104 Pool
- **Tax year**: 6 April to 5 April (e.g., 2024/25 = 6 Apr 2024 – 5 Apr 2025)
- **Reference**: `docs/tax-rules.md`, `docs/spec.md`, HMRC CG51500-CG51600

## Never

- Remove or modify tests without proving incorrectness
- Commit code that fails `cargo clippy` or `cargo test`
- Guess tax calculations — verify against `docs/tax-rules.md`
- Include internal monologue or self-dialogue in comments; comments must state facts, constraints, or rationale

## Commits

Format: `type: description` (feat/fix/test/docs/chore/refactor)

Run `cargo fmt && cargo clippy` before committing.

## Release

See `docs/release.md` for the full release procedure.
