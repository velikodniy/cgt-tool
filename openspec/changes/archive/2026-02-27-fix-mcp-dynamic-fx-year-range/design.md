## Context

The MCP server's `get_fx_rate` tool checks whether a currency exists in the FX cache by scanning a hardcoded year range `(2015..=2025)`. This was a reasonable shortcut when all bundled data fell within that range, but becomes a bug as new years of FX data are added — the check will incorrectly report "unknown currency" for valid currencies with data only in years beyond 2025.

The `FxCache` currently has `get`, `insert`, `extend`, `len`, and `is_empty` methods but no way to query which currencies exist across all periods.

## Goals / Non-Goals

**Goals:**

- Derive currency existence from actual cache contents, not a static range
- Keep the fix minimal — one new method on `FxCache`, one line changed in MCP server

**Non-Goals:**

- Returning the list of available years per currency (not needed for this fix)
- Changing how rates are loaded or bundled
- Modifying any FX conversion logic

## Decisions

**Add `FxCache::has_currency` method**: Iterate over cached `RateKey` entries to check if any entry matches the given currency code. This is a simple `any()` over the existing `HashMap` keys.

Alternative considered: storing a separate `HashSet<Currency>` of known currencies. Rejected because `has_currency` is only called in an error path (rate not found), so the O(n) scan of the hashmap is fine — correctness matters here, not micro-optimization.

Alternative considered: widening the hardcoded range (e.g., `2015..=2099`). Rejected because it still encodes assumptions about the data and is fragile.

## Risks / Trade-offs

- \[Performance of `has_currency`\] → Only called in error path when `get()` already returned `None`. The cache has ~1300 entries; iterating is negligible.
- [Public API surface] → Adding a public method to `FxCache` is a minor expansion, but it's a natural query for the cache to support.
