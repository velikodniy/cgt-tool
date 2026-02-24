## Context

The `validate()` function in `crates/cgt-core/src/validation.rs` validates transactions before CGT calculation. The Buy, Sell, and CapReturn match arms each contain identical checks for zero quantity, negative quantity, negative price/total_value, and negative fees. This duplication spans ~120 lines of the 270-line function body.

## Goals / Non-Goals

**Goals:**

- Extract common trade field checks into a private `check_trade_fields()` helper
- Reduce validation code by ~80 lines
- Maintain identical validation behavior (same errors, same messages)

**Non-Goals:**

- Changing validation rules or error messages
- Refactoring Dividend, Split, or Unsplit arms (Dividend has `total_value` but no `fees`; structure differs enough to not benefit)
- Changing public API (`validate()`, `ValidationResult`, error/warning types)

## Decisions

### Extract `check_trade_fields` as a private function

The helper takes the common fields shared by Buy, Sell, and CapReturn: `result`, `line`, `date`, `ticker`, `action` label, `amount`, price/total_value amount, and fees amount. It pushes errors to the mutable `ValidationResult`.

**Rationale**: These three operations share identical checks for quantity (zero/negative) and monetary fields (negative price, negative fees). The `action` parameter (e.g., "BUY") is passed as a string to preserve the current error message format.

**Alternative considered**: Using a trait or generic validation — rejected as overengineered for four simple checks.

**Field mapping**: Buy/Sell pass `price.amount` for the price field; CapReturn passes `total_value.amount`. Both map to the same "price" concept in validation terms. The helper accepts a `price_label` parameter ("price" vs "total value") to preserve exact error messages.

## Risks / Trade-offs

- [Slight indirection] The helper adds a function call layer. → Acceptable given the large duplication reduction.
- [Message fidelity] Error messages must remain identical. → The `action` and `price_label` parameters preserve exact wording.
