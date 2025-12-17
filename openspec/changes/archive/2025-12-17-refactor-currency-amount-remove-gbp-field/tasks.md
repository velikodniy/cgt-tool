# Tasks: Remove gbp field from CurrencyAmount

## 1. Core Type Changes (cgt-money)

- [x] 1.1 Add `to_gbp(&self, date, fx_cache) -> Result<Decimal>` method to `CurrencyAmount` (already done)
- [x] 1.2 Update all internal usages of `.gbp` to use `to_gbp()` method (already done)
- [x] 1.3 Remove `gbp` field from `CurrencyAmount` struct (already done)
- [x] 1.4 Remove `CurrencyAmount::gbp()` helper; use a single `new(amount, currency)` constructor (already done)
- [x] 1.5 Rename `CurrencyAmount::foreign()` to `new()` with only amount + currency (already done)
- [x] 1.6 Update `Deserialize` impl to reject legacy `gbp` input
- [x] 1.7 Update `Serialize` impl to omit `gbp` field (already done)
- [x] 1.8 Update `JsonSchema` impl (already done)
- [x] 1.9 Update tests in cgt-money (already done)

## 2. Parser Changes (cgt-core)

- [x] 2.1 Remove `parse_file_with_fx` entirely; parsing never does FX (already done - not present)
- [x] 2.2 Update `parse_money` to drop unused `_date` and `_ctx` args
- [x] 2.3 Simplify `parse_file` to not need FX cache (already done)
- [x] 2.4 Update parser tests (already passing)

## 3. Calculation Changes (cgt-core)

- [x] 3.1 Add `fx_cache` parameter to `calculate()` function (already done)
- [x] 3.2 Make `Operation<M>` generic and introduce `GbpTransaction` with `Operation<Decimal>`
- [x] 3.3 Add `Transaction::to_gbp()` and `transactions_to_gbp()` conversion functions
- [x] 3.4 Update calculator to use `GbpTransaction` throughout (no in-place mutation)
- [x] 3.5 Update same-day matching to use GBP amounts
- [x] 3.6 Update bed-and-breakfast matching to use GBP amounts
- [x] 3.7 Update Section 104 pool calculations to use GBP amounts
- [x] 3.8 Update calculation tests (all passing)

## 4. CLI Changes (cgt-cli)

- [x] 4.1 Pass FX cache to `calculate()` calls (already done)
- [x] 4.2 Update command handlers (already done)
- [x] 4.3 Update CLI tests (all passing)

## 5. Formatter Changes

- [x] 5.1 Update `cgt-format` to handle amounts without pre-computed GBP (no changes needed)
- [x] 5.2 Update `cgt-formatter-plain` (no changes needed)
- [x] 5.3 Update `cgt-formatter-pdf` (no changes needed)

## 6. MCP Server Changes (cgt-mcp)

- [x] 6.1 Simplify JSON input parsing (no `gbp` field needed) - N/A, cgt-mcp not present
- [x] 6.2 Pass FX cache to calculation - N/A
- [x] 6.3 Update tool descriptions and examples - N/A
- [x] 6.4 Update SERVER_INSTRUCTIONS - N/A
- [x] 6.5 Update MCP tests - N/A

## 7. Integration & Validation

- [x] 7.1 Run full test suite (234 tests passing)
- [x] 7.2 Verify JSON output format (no `gbp` field - already done)
- [x] 7.3 Test MCP server with simplified JSON input - N/A
- [x] 7.4 Update snapshot tests if output format changes (no changes needed)
