## Context

`FxCache::get` currently accepts `&str` and normalizes it via `trim().to_uppercase()` on every call, allocating a `String` each time. The `Currency` enum from `iso_currency` already guarantees valid, uppercase ISO 4217 codes. All production callers have a `Currency` value available before calling `get`.

## Goals / Non-Goals

**Goals:**

- Eliminate per-lookup `String` allocation in `FxCache::get`
- Strengthen the API by accepting `Currency` instead of `&str`
- Keep the change minimal and mechanical

**Non-Goals:**

- Optimizing other parts of the FX lookup path
- Changing the `FxCache` storage format or key structure

## Decisions

**Accept `Currency` instead of `&str`**

The `Currency` enum already guarantees a valid, uppercase ISO 4217 code. Accepting it directly:

- Eliminates the `trim().to_uppercase()` allocation
- Removes the `Currency::from_code` parse step (which can return `None`)
- Makes invalid currency codes a compile-time or caller-side concern rather than a silent `None`

Alternative considered: checking if the string is already uppercase before allocating. This would still require the `Currency::from_code` parse and doesn't improve type safety.

Alternative considered: using `Cow<str>`. This adds complexity without the type-safety benefit.

**Callers handle their own parsing**

The MCP server receives raw user strings and must parse `Currency::from_code` itself before calling `get`. This is appropriate since the MCP server already validates and normalizes user input.

## Risks / Trade-offs

- [Breaking public API] `FxCache::get` signature changes. -> All callers are within this workspace and are updated together.
- [Loss of case-insensitive lookup] Tests that pass lowercase strings will need updating. -> This convenience was unused in production; all real callers pass `Currency` values which are always correctly cased.
