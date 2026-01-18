# Change: Refactor Schwab converter for JSON-only inputs

## Why

Schwab exports are now provided as JSON for both transactions and equity awards, and the converter must align to the current JSON structure while remaining HMRC-compliant and IO-free.

## What Changes

- **BREAKING**: Schwab converter accepts JSON inputs for transactions and equity awards only; CSV inputs are no longer supported.
- Document the Schwab JSON structures in a dedicated doc and relocate tax rules documentation into the docs area (review and clarify tax rules wording if needed).
- Update CLI and MCP documentation to reflect JSON-only Schwab conversion inputs.
- Update converter behavior to map Schwab JSON fields to CGT DSL using HMRC guidance on RSU vesting dates and acquisition costs.
- Update Schwab test fixtures to JSON.

## Impact

- Affected specs: `broker-conversion`, `cli`, `mcp-server`
- Affected code: `crates/cgt-converter/src/schwab/mod.rs`, `crates/cgt-converter/src/schwab/awards.rs`, `crates/cgt-cli/src/commands.rs`, `crates/cgt-mcp/src/resources.rs`, `crates/cgt-mcp/src/server.rs`
- Affected docs: `README.md`, `AGENTS.md`, `TAX_RULES.md` (relocated), new Schwab JSON structure doc under `docs/`
