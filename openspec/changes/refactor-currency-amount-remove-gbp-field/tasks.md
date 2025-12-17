# Tasks: Remove gbp field from CurrencyAmount

## 1. Core Type Changes (cgt-money)

- [ ] 1.1 Add `to_gbp(&self, date, fx_cache) -> Result<Decimal>` method to `CurrencyAmount`
- [ ] 1.2 Update all internal usages of `.gbp` to use `to_gbp()` method
- [ ] 1.3 Remove `gbp` field from `CurrencyAmount` struct
- [ ] 1.4 Remove `CurrencyAmount::gbp()` helper; use a single `new(amount, currency)` constructor (defaults to GBP where needed)
- [ ] 1.5 Rename `CurrencyAmount::foreign()` to `new()` with only amount + currency
- [ ] 1.6 Update `Deserialize` impl to not require `gbp` field and reject legacy `gbp` input
- [ ] 1.7 Update `Serialize` impl to omit `gbp` field (breaking output OK)
- [ ] 1.8 Update `JsonSchema` impl
- [ ] 1.9 Update tests in cgt-money

## 2. Parser Changes (cgt-core)

- [ ] 2.1 Remove `parse_file_with_fx` entirely; parsing never does FX
- [ ] 2.2 Update `parse_currency_amount`/`parse_money` to return unconverted amounts and drop unused args
- [ ] 2.3 Simplify `parse_file` to not need FX cache
- [ ] 2.4 Update parser tests

## 3. Calculation Changes (cgt-core)

- [ ] 3.1 Add `fx_cache` parameter to `calculate()` function
- [ ] 3.2 Introduce `GbpTransaction` (keeping Operation shape generic) and convert at calculation entry
- [ ] 3.3 Update matcher to operate on GBP-normalized data (no in-place mutation of `Transaction`)
- [ ] 3.4 Update same-day matching to use GBP amounts
- [ ] 3.5 Update bed-and-breakfast matching to use GBP amounts
- [ ] 3.6 Update Section 104 pool calculations to use GBP amounts
- [ ] 3.7 Update calculation tests

## 4. CLI Changes (cgt-cli)

- [ ] 4.1 Pass FX cache to `calculate()` calls
- [ ] 4.2 Update command handlers
- [ ] 4.3 Update CLI tests

## 5. Formatter Changes

- [ ] 5.1 Update `cgt-format` to handle amounts without pre-computed GBP
- [ ] 5.2 Update `cgt-formatter-plain`
- [ ] 5.3 Update `cgt-formatter-pdf`

## 6. MCP Server Changes (cgt-mcp)

- [ ] 6.1 Simplify JSON input parsing (no `gbp` field needed)
- [ ] 6.2 Pass FX cache to calculation
- [ ] 6.3 Update tool descriptions and examples
- [ ] 6.4 Update SERVER_INSTRUCTIONS
- [ ] 6.5 Update MCP tests

## 7. Integration & Validation

- [ ] 7.1 Run full test suite
- [ ] 7.2 Verify JSON output format (no `gbp` field)
- [ ] 7.3 Test MCP server with simplified JSON input
- [ ] 7.4 Update snapshot tests if output format changes
