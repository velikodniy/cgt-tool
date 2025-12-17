# Change: Remove redundant `gbp` field from CurrencyAmount

## Why

The current `CurrencyAmount` struct stores three fields: `amount`, `currency`, and `gbp`. The `gbp` field is problematic:

1. **Redundant data**: For GBP currency, `gbp` always equals `amount`
2. **Correlated values**: For foreign currencies, `gbp` is derived from `amount` using FX rates - storing both allows inconsistency
3. **Poor API for JSON input**: External callers (like LLMs via MCP) must pre-compute the GBP conversion, which requires them to:
   - Know about FX conversion
   - Call `get_fx_rate` first
   - Calculate `gbp = amount / rate`
   - Risk providing inconsistent values

## What Changes

- **BREAKING**: Remove `gbp` field from `CurrencyAmount` struct; add `to_gbp(&self, date, fx_cache) -> Result<Decimal>`
- Simplify constructors: remove `gbp()` helper; rename `foreign()` to `new()` that takes only amount + currency
- Simplify JSON to accept only `amount` and optional `currency` (default GBP); `gbp` not emitted in output; reject legacy `gbp` field on input
- Thread FX cache into calculations and compute GBP on-demand using transaction date
- Introduce GBP-normalized view (`GbpTransaction`) instead of mutating `Transaction` values in-place during calculation

## Impact

- Breaking JSON output is acceptable (0.x.y)
- Affected specs: `cgt-money`
- Affected code:
  - `crates/cgt-money/src/amount.rs` - Core type change
  - `crates/cgt-core/src/matcher/*.rs` - Update to use `to_gbp()` method
  - `crates/cgt-core/src/calculator.rs` - Update to use `to_gbp()` method
  - `crates/cgt-core/src/parser.rs` - Simplify parsing (no FX at parse time)
  - `crates/cgt-mcp/src/server.rs` - Simplified JSON input
  - `crates/cgt-format/src/lib.rs` - Update formatting
  - `crates/cgt-formatter-*/src/lib.rs` - Update formatters
