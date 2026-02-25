## Context

The `Matcher` struct in `crates/cgt-core/src/matcher/mod.rs` exposes two internal helper methods (`get_ledger_mut`, `get_pool_mut`) as `pub`. These are only called by sibling submodules `same_day.rs` and `section104.rs` via `super::Matcher`. No code outside the `matcher` module uses them.

## Goals / Non-Goals

**Goals:**

- Restrict `get_ledger_mut` and `get_pool_mut` to `pub(super)` visibility
- Enforce encapsulation of matcher internals at the module boundary

**Non-Goals:**

- Changing any matcher behavior or calculation logic
- Refactoring how submodules access the matcher
- Auditing visibility of other structs or methods beyond these two

## Decisions

- **Use `pub(super)` over `pub(crate)`**: These methods are only needed by sibling submodules within `matcher/`. `pub(super)` is the tightest visibility that satisfies all callers. `pub(crate)` would be unnecessarily broad.
- **No other methods changed**: Review of the `Matcher` impl block confirms all other `pub` methods (`new`, `process`) are part of the external API used by callers outside the matcher module.

## Risks / Trade-offs

- [Risk] Future code outside the matcher module might need these methods → Add explicit `pub` or `pub(crate)` at that time with justification. The compiler will flag the visibility error immediately.
- No migration needed — this is a non-breaking internal change.
