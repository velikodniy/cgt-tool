## Context

The codebase has two implementations of CGT share matching:

1. **`calculator.rs`** (629 lines) - Monolithic, currently used
2. **`matcher/`** (5 files, ~850 lines) - Modular, unused

Both implement the same HMRC matching rules (Same Day, B&B, Section 104). The matcher module was designed for O(n) efficiency using acquisition ledgers but was never integrated.

## Goals / Non-Goals

**Goals:**

- Use the better-structured matcher module
- Remove duplicate code from calculator.rs
- Maintain identical behavior (all tests must pass)

**Non-Goals:**

- Changing the public API of `calculate()`
- Performance optimization (current performance is adequate)
- Adding new matching features

## Decisions

### Decision: Adapt Matcher to GbpTransaction

The matcher currently uses `Transaction` with `CurrencyAmount` fields. The calculator converts to `GbpTransaction` (with plain `Decimal` prices) early in processing.

**Approach:** Modify matcher to accept `GbpTransaction` directly, avoiding double conversion.

**Alternatives considered:**

- Keep Transaction in matcher, convert back and forth → More complex, error-prone
- Remove GbpTransaction entirely → Larger scope, affects FX conversion logic

### Decision: Keep Calculator as Entry Point

The `calculate()` function remains the public API. It will:

1. Convert transactions to GBP
2. Delegate matching to `Matcher`
3. Filter by tax year
4. Group matches into disposals

This preserves the existing interface while using the modular internals.

## Risks / Trade-offs

- **Risk:** Subtle behavioral differences between implementations

  - **Mitigation:** 34 golden file tests cover all matching scenarios

- **Risk:** Matcher may have bugs not caught by current tests

  - **Mitigation:** The matcher was designed from the same spec; any divergence is a bug to fix

## Migration Plan

1. Adapt matcher module to GbpTransaction
2. Incrementally replace calculator passes with matcher calls
3. Run tests after each step to catch regressions
4. Remove dead code once all tests pass

No rollback needed - this is internal refactoring with no external API changes.

## Open Questions

None - the implementation path is clear.
