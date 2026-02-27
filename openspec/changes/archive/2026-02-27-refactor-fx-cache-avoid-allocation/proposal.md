## Why

`FxCache::get` accepts `&str` and calls `code.trim().to_uppercase()` on every lookup, allocating a new `String` each time. All production callers already have a `Currency` value available (from `CurrencyAmount::currency` or parsed upstream), making this normalization redundant and wasteful on hot paths. Accepting `Currency` directly eliminates the allocation and provides stronger type safety.

## What Changes

- Change `FxCache::get` signature from `(&self, code: &str, year: i32, month: u32)` to `(&self, currency: Currency, year: i32, month: u32)`
- Remove the `trim().to_uppercase()` allocation and `Currency::from_code` parsing inside `get`
- Update all callers in `cgt-money` (amount.rs), `cgt-mcp` (server.rs), and tests to pass `Currency` directly
- Remove the case-insensitive lookup behavior (callers must provide a valid `Currency` enum value)

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `cgt-money`: `FxCache::get` signature changes from `&str` to `Currency` parameter. This is a **BREAKING** API change for the crate's public interface.

## Impact

- **cgt-money**: `FxCache::get` signature change, `CurrencyAmount::to_gbp` updated to pass `self.currency` directly
- **cgt-mcp**: `server.rs` callers updated to parse `Currency` from user input before calling `get`
- **Tests**: `fallback_tests.rs` and `parser_cache_tests.rs` updated to pass `Currency` values instead of string literals
- **No behavioral change**: All production code paths produce identical results; only the case-insensitive string lookup convenience is removed
