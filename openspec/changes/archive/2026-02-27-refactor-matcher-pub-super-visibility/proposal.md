## Why

`Matcher::get_ledger_mut` and `Matcher::get_pool_mut` are declared `pub` but are only called by sibling submodules (`same_day.rs`, `section104.rs`). This exposes internal mutation to any external caller, violating the Deep Modules principle of hiding implementation details behind simple interfaces.

## What Changes

- Restrict `get_ledger_mut` from `pub` to `pub(super)` visibility
- Restrict `get_pool_mut` from `pub` to `pub(super)` visibility
- No behavioral changes; this is a purely additive encapsulation improvement

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none -- this is an internal visibility refactor with no spec-level behavior changes)

## Impact

- `crates/cgt-core/src/matcher/mod.rs`: Visibility modifiers on two methods change
- No external API changes (methods were not used outside the matcher module)
- No test changes required (tests do not call these methods directly)
