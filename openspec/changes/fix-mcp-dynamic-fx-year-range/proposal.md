## Why

The MCP server's `get_fx_rate` tool uses a hardcoded year range `(2015..=2025)` to check whether a currency exists in the FX cache. After 2025, this will always report "unknown currency" instead of "missing FX rate" for valid currencies, producing misleading error messages. The range should be derived from actual cache contents.

## What Changes

- Add a method to `FxCache` that checks whether a currency code exists in any cached period
- Replace the hardcoded `(2015..=2025)` range in the MCP server with a call to this new method
- Error messages for missing FX rates will remain accurate regardless of which years are loaded

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `cgt-money`: Add `has_currency` method to `FxCache` for checking currency existence across all cached periods
- `mcp-server`: Use `FxCache::has_currency` instead of hardcoded year range for currency existence check

## Impact

- `crates/cgt-money/src/cache.rs`: New public method on `FxCache`
- `crates/cgt-mcp/src/server.rs`: Replace hardcoded range with `has_currency` call (~1 line change)
- No breaking changes to public APIs or CLI behavior
