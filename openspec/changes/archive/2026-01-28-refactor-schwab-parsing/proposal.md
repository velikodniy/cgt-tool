## Why

The current Schwab parser is brittle and prone to failure when encountering unknown transaction types that lack expected fields, even if those transactions are irrelevant for tax calculations. Additionally, the parser currently emits warnings for every skipped transaction, polluting the output and creating noise for users.

## What Changes

- Refactor `crates/cgt-converter/src/schwab` to use `serde`'s polymorphic deserialization capabilities (Algebraic Data Types).
- Define strict structs for known transaction types (`Buy`, `Sell`, `Stock Plan Activity`, `Cash Dividend`, etc.) based on the `Action` field.
- Implement a fallback mechanism to safely capture unknown or irrelevant transactions without causing parsing errors.
- Treat unknown or irrelevant transactions as ignorable without special cases.
- Update the reporting logic to silence warnings for known-skipped transactions, instead adding them as comments in the generated `.cgt` output.

## Capabilities

### New Capabilities

<!-- Capabilities being introduced. Replace <name> with kebab-case identifier (e.g., user-auth, data-export, api-rate-limiting). Each creates specs/<name>/spec.md -->

### Modified Capabilities

<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes. Each needs a delta spec file.
     Use existing spec names from openspec/specs/. Leave empty if no requirement changes. -->

## Impact

- `crates/cgt-converter/src/schwab/mod.rs`
- `crates/cgt-converter/src/schwab/awards.rs`
- `crates/cgt-converter/src/schwab/transactions.rs` (likely needed to split logic)
