## 1. Add `has_currency` to FxCache

- [x] 1.1 Add `has_currency(&self, code: &str) -> bool` method to `FxCache` in `crates/cgt-money/src/cache.rs`
- [x] 1.2 Add unit tests for `has_currency` (valid currency, unknown currency, case-insensitive, invalid ISO code)

## 2. Update MCP server to use dynamic check

- [x] 2.1 Replace hardcoded `(2015..=2025)` range in `get_fx_rate` with `cache.has_currency()` call
- [x] 2.2 Run `cargo fmt && cargo clippy && cargo test` and verify all checks pass
