## Context

The oversell guard in `process_sell` (`matcher/mod.rs:384`) checks whether `remaining > 0` after the matching cascade (Same Day → B&B → S104). This catches sells that can't be fully matched, but it runs too late — B&B matching can satisfy the entire sell quantity by reserving a future buy, even when the seller holds zero shares. The result is a silently accepted disposal of shares the taxpayer never held.

Per HMRC CG51590 Example 1, B&B determines cost basis for a disposal that is already valid (the taxpayer had shares in the S104 pool). B&B does not create a holding out of nothing.

The converter (`convert-to-raw.py`) also emits transactions in file order without sorting, causing cgt-calc to crash on valid files where SELL appears before BUY on the same day (e.g., `CarryLoss.cgt`).

## Goals / Non-Goals

**Goals:**

- Reject SELL transactions where the holding (same-day ledger + S104 pool) is insufficient, before matching begins.
- Fix the `MultipleMatches.cgt` fixture to test all three matching rules with valid holdings.
- Fix the converter to sort output so cgt-calc can process same-day SELL-before-BUY ordering.
- Eliminate 2 of the 5 cross-validation discrepancies (CarryLoss, MultipleMatches).

**Non-Goals:**

- Short selling support. HMRC CGT rules assume the taxpayer holds shares before disposing.
- Changing the matcher's processing order (buys before sells within a day). This is already correct and matches HMRC expectations.
- Modifying cgt-core's sort behavior. The `.cgt` format expects date-ordered input; cgt-core sorts by date with a stable sort. The converter is a separate tool.

## Decisions

### Decision 1: Pre-cascade holding check in `process_sell`

Add a holding check at the top of `process_sell`, before any matching occurs:

```
holding = ledger.remaining_for_ticker(ticker) + pool.quantity(ticker)
if sell_amount > holding → error
```

**Why not check inside B&B?** The B&B module shouldn't know about holdings — it only looks for future acquisitions. The invariant "you must hold what you sell" belongs at the sell entry point.

**Why not use the existing post-cascade check?** It only catches the case where matching rules can't find enough shares. It cannot distinguish "no shares held" from "matched via B&B from a future buy." The pre-check is the correct place because it validates the precondition, not the postcondition.

**Ledger remaining vs. pool:** Same-day buys are in the ledger (not yet moved to pool). Prior-day buys have been moved to the pool. The holding is the sum of both. `remaining_for_ticker` doesn't exist yet — we need a method that sums remaining shares across all dates, or we query `remaining_for_date` for the current date plus the pool.

Actually, the ledger only holds same-day lots (prior-day lots are moved to pool at end of each day). So the check is: `ledger.remaining_for_date(sell_date) + pool.quantity`. This is simpler — same-day buys are in the ledger, everything else is in the pool.

### Decision 2: Rewrite `MultipleMatches.cgt` with valid holdings

The current fixture sells 10 shares on 2019-08-28 with 0 holding. Rewrite to maintain a valid holding throughout. The fixture should still exercise all three matching rules (Same Day, B&B, S104) in separate tax years with independent buy/sell pairs that each have sufficient holdings.

Simplest approach: ensure each B&B scenario has a prior pool holding that covers the sell. The B&B buy within 30 days then provides the cost basis (per CG51590 Example 1), but the shares being sold come from the pool.

### Decision 3: Sort converter output by date

Sort the RAW CSV output from `convert-to-raw.py` by `(date, action)` where BUY sorts before SELL. This is a Python-side change only, not a cgt-core change. The `.cgt` parser's stable date sort is unaffected.

**Why BUY before SELL?** cgt-calc processes transactions sequentially and requires shares to exist before selling. Within a day, BUY before SELL is the natural order for any tool that doesn't aggregate same-day transactions.

## Risks / Trade-offs

**[Breaking change]** → Input files that previously produced results will now error if they contain sells without backing holdings. This is correct — those results were wrong. The error message will explain the issue clearly.

**[Test fixture rewrite]** → Changing `MultipleMatches.cgt` changes golden files. The new fixture must still exercise all three rules to maintain coverage. → Verify the new fixture exercises Same Day, B&B, and S104 matching by inspecting the output.

**[Converter sort may mask input issues]** → Sorting converter output hides the fact that the source `.cgt` file has non-chronological entries within a day. → Acceptable because the converter's job is to produce valid input for cgt-calc, and cgt-calc doesn't sort internally. The `.cgt` file itself is processed by cgt-tool which handles same-day ordering correctly.
