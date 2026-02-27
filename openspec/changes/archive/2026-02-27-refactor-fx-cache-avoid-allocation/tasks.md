## 1. Change FxCache::get signature

- [x] 1.1 Change `FxCache::get` in `crates/cgt-money/src/cache.rs` to accept `Currency` instead of `&str`, removing `trim().to_uppercase()` and `Currency::from_code`

## 2. Update callers

- [x] 2.1 Update `CurrencyAmount::to_gbp` in `crates/cgt-money/src/amount.rs` to pass `self.currency` directly
- [x] 2.2 Update MCP server callers in `crates/cgt-mcp/src/server.rs` to parse `Currency` before calling `get`

## 3. Update tests

- [x] 3.1 Update `crates/cgt-money/tests/fallback_tests.rs` to use `Currency` values instead of string literals
- [x] 3.2 Update `crates/cgt-money/tests/parser_cache_tests.rs` to use `Currency` values instead of string literals

## 4. Verify

- [x] 4.1 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass
