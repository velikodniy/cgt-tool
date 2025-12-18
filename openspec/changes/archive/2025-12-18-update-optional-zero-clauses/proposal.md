# Change: Make TAX and FEES clauses optional in DSL

## Why

Users must currently write `TAX 0` on dividends and `FEES 0` on transactions even when values are zero. This adds verbosity without value.

## What Changes

- Make `TAX` clause optional for DIVIDEND commands (defaults to 0)
- Document that `FEES` is already optional for BUY/SELL/CAPRETURN (defaults to 0)
- Update spec scenarios to clarify optional behavior

## Impact

- Affected specs: dsl-parsing
- Affected code: `crates/cgt-core/src/parser.pest`, parsing logic in `crates/cgt-core/src/`, `crates/cgt-mcp/src/resources.rs`, `crates/cgt-mcp/src/server.rs`
