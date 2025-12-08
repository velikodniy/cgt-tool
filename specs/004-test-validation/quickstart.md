# Test Verification Quickstart Guide

**Purpose:** Step-by-step guide for manually verifying CGT test calculations

**Prerequisites:**

- Read TAX_RULES.md (will be created in project root)
- Read research.md (HMRC rules summary)
- Calculator or spreadsheet for calculations
- Text editor for adding comments to .cgt files

---

## Overview

This guide explains how to manually verify CGT calculations in test files, add verification comments, and resolve discrepancies.

### Verification Levels

**Simple Verification** (for most tests):

- Add header comment with test metadata
- Review transactions and verify matching logic makes sense
- Quick mental/calculator check of gain/loss
- Add minimal inline comments for clarity

**Detailed Verification** (for 5+ priority tests):

- Everything in simple verification
- Step-by-step written calculations
- Document every matching decision
- Show pool maintenance at each step
- Detailed inline comments explaining each transaction

---

## Simple Verification Process

### Step 1: Open Test File

```bash
# Example
open tests/data/Simple.cgt
```

### Step 2: Read Transactions

Understand what the test does:

- How many transactions?
- Which asset(s)?
- Which tax year(s)?
- Any special events (splits, dividends, capital returns)?

### Step 3: Identify Expected Matching Rule

Based on transaction dates and pattern:

- **Same Day**: Buy and sell on same date
- **Bed & Breakfast**: Sell followed by buy within 30 days
- **Section 104**: Default pooling for other transactions

### Step 4: Quick Verification

Mental or calculator check:

- For Same Day: Proceeds - Cost - Expenses = Gain/Loss
- For B&B: Same calculation with 30-day window check
- For Section 104: Check pool average cost makes sense

### Step 5: Add Header Comment

Copy this template to top of .cgt file:

```
# Test: [Filename without extension]
# Purpose: [One sentence describing what this validates]
# Rules Tested: [Same Day | Bed & Breakfast | Section 104 | Multiple]
# Complexity: [Simple | Complex]
# Key Features: [e.g., "Basic same-day buy-sell", "Multi-year pool"]
# Expected Outcome: [e.g., "Gain £520.00"]
#
# Verification Status: Verified
# Verified By: [Your name], [Date]
# Verification Notes: [Brief note, e.g., "Simple same-day match, straightforward calculation"]
```

### Step 6: Add Inline Comments (if needed)

For simple tests, minimal comments:

```
# Same Day match: Buy 10 @ £4.15, Sell 10 @ £4.67
2018-08-28 BUY GB00B41YBW71 10 @ 4.1565 EXPENSES 12.5
2018-08-28 SELL GB00B41YBW71 10 @ 4.6702 EXPENSES 12.5
# Gain: (10 × 4.6702) - (10 × 4.1565) - (12.5 + 12.5) = £5.14
```

### Step 7: Verify Against Expected Output

```bash
# Check expected gain/loss in .json file
cat tests/data/Simple.json
```

Compare your calculation with `total_gain` and `total_loss` in .json.

### Step 8: Run Tests

```bash
cargo test
```

Ensure test still passes.

---

## Detailed Verification Process

Use for these priority tests:

1. HMRCExample1.cgt
2. WithAssetEventsBB.cgt
3. WithAssetEventsMultipleYears.cgt
4. MultipleMatches.cgt
5. SameDayMerge.cgt
6. CarryLoss.cgt

### Step 1-4: Same as Simple Verification

### Step 5: Create Calculation Workspace

Use spreadsheet or paper with columns:

- Date
- Action (BUY/SELL)
- Quantity
- Price
- Expenses
- Rule Applied
- Cost Basis
- Proceeds
- Gain/Loss
- Pool Quantity After
- Pool Cost After

### Step 6: Apply Matching Rules Step-by-Step

For each SELL transaction, determine matches in order:

**Priority 1: Same Day**

- Check if any BUY on same date
- If yes, match quantities (FIFO if multiple buys)
- Calculate gain/loss
- Remaining sell quantity goes to next rule

**Priority 2: Bed & Breakfast (30-day)**

- Check purchases in next 30 days after sell date
- Match in chronological order (earliest buy first)
- Calculate gain/loss for B&B matched quantity
- Remaining sell quantity goes to Section 104

**Priority 3: Section 104 Pool**

- Use current pool average cost
- Calculate gain/loss: (Sell Price × Qty) - (Pool Avg Cost × Qty) - Expenses
- Update pool: Remove sold quantity at average cost

For each BUY transaction:

- If matched via Same Day or B&B: Don't add to pool
- Otherwise: Add to pool (update quantity and total cost, recalculate average)

### Step 7: Document Each Transaction

Add detailed comments inline:

```
# === DETAILED VERIFICATION ===
# Tax Year: 2018/2019
#
# Transaction 1: 2018-04-10 BUY 100 @ £5.00
#   Rule: Enters Section 104 pool (no matching sell)
#   Cost Basis: 100 × £5.00 + £10 expenses = £510.00
#   Pool After: 100 shares @ £5.10 avg cost (£510 total)
#
2018-04-10 BUY AAPL 100 @ 5.00 EXPENSES 10.00
#
# Transaction 2: 2018-05-15 BUY 50 @ £6.00
#   Rule: Enters Section 104 pool
#   Cost Basis: 50 × £6.00 + £5 expenses = £305.00
#   Pool After: 150 shares @ £5.43 avg cost (£815 total)
#   Calculation: (£510 + £305) / (100 + 50) = £5.43
#
2018-05-15 BUY AAPL 50 @ 6.00 EXPENSES 5.00
#
# Transaction 3: 2018-06-20 SELL 75 @ £7.00
#   Rule: Section 104 (no same-day or B&B match)
#   Matched Against: Pool at £5.43 avg cost
#   Proceeds: 75 × £7.00 - £8 expenses = £517.00
#   Cost: 75 × £5.43 = £407.25
#   Gain: £517.00 - £407.25 = £109.75
#   Pool After: 75 shares @ £5.43 avg cost (£407.75 total)
#   Calculation: £815 - £407.25 = £407.75
#
2018-06-20 SELL AAPL 75 @ 7.00 EXPENSES 8.00
#
# FINAL RESULT:
#   Total Gain: £109.75
#   Total Loss: £0.00
#   Verification: ✓ Matches expected output in .json
# === END DETAILED VERIFICATION ===
```

### Step 8: Cross-Check with HMRC Guidance

Verify your approach against TAX_RULES.md:

- Same Day rule applied correctly?
- B&B 30-day window calculated properly?
- Pool average cost formula correct?
- Tax year boundaries respected?

### Step 9: Compare with Expected Output

```bash
cat tests/data/[TestName].json
```

Check:

- Total gain matches
- Total loss matches
- Individual match records make sense

### Step 10: Document Completion

Update header comment:

```
# Verification Status: Verified
# Verified By: [Name], 2025-12-08
# Verification Notes: Detailed step-by-step verification complete. All calculations match expected output. HMRC guidance CG51575 (Section 104) followed.
```

---

## Discrepancy Resolution Workflow

### When Manual Calculation ≠ Expected Output

**Step 1: Document the Discrepancy**

Add comment to test file:

```
# DISCREPANCY FOUND:
#   Our calculation: Gain £520.00
#   Expected (.json): Gain £515.00
#   Difference: £5.00
#   Investigation: [Date]
```

**Step 2: Double-Check Your Calculation**

Common mistakes:

- Forgot to include expenses?
- Used wrong pool average cost?
- Applied wrong matching rule?
- Math error?

Recalculate carefully.

**Step 3: Consult HMRC Guidance**

Check TAX_RULES.md and HMRC manual:

- Same Day: CG51560
- B&B: CG51560
- Section 104: CG51575
- Examples: CG51590

Find official guidance on the specific scenario.

**Step 4: Determine Root Cause**

Possibilities:

**A) Our Code Bug**

- Calculator has incorrect matching logic
- Pool calculation error
- Expense handling wrong
- Date parsing issue

Action: Fix code, document fix, re-run all tests

**B) Our Test Bug**

- .json file has incorrect expected values
- Values copied incorrectly from cgtcalc
- Test created with buggy calculator

Action: Update .json with proof from HMRC, document justification

**C) cgtcalc Difference**

- cgtcalc may have different interpretation
- Check if cgtcalc result also differs from HMRC

Action: Follow HMRC guidance as authority, document our approach

**D) Rounding/Precision**

- Difference ≤ £1.00 (acceptable tolerance per SC-006)

Action: Document as acceptable rounding difference, no fix needed

**Step 5: Take Action**

**If Code Bug:**

```bash
# Fix bug in calculator.rs or parser.rs
# Example commit message:
git add crates/cgt-core/src/calculator.rs
git commit -m "fix: correct Section 104 pool average cost calculation

Previous implementation divided total cost by current quantity instead
of updating incrementally. This caused incorrect average cost when
multiple acquisitions occurred.

Fixed per HMRC CG51575 guidance on pool maintenance.

Verified against HMRCExample1.cgt and WithAssetEventsBB.cgt tests."
```

**If Test Bug (requires HMRC proof):**

```bash
# Update .json file ONLY with documented proof
# Add justification to test file:

# EXPECTED VALUE CORRECTION:
#   Previous: Gain £515.00
#   Corrected: Gain £520.00
#   Justification: HMRC CG51575 Example 2 shows that capital returns
#                  reduce pool cost before disposal, not after.
#                  Original .json value was incorrect.
#   HMRC Reference: CG51575, paragraph 12
#   Verified: 2025-12-08

git add tests/data/WithAssetEventsBB.json tests/data/WithAssetEventsBB.cgt
git commit -m "fix: correct expected gain in WithAssetEventsBB test

Previous expected value of £515 was incorrect. Capital returns must
reduce pool cost before calculating disposal gain, not after.

Corrected to £520 per HMRC CG51575 guidance.

Added detailed verification comments to .cgt file with HMRC reference."
```

**Step 6: Re-verify**

```bash
cargo test
```

Ensure:

- All tests pass
- Manual calculation now matches
- No regressions introduced

**Step 7: Update Verification Status**

```
# Verification Status: Verified
# Discrepancy Resolved: [Date]
# Resolution: [Brief description of fix]
# HMRC Reference: CG[section]
```

---

## Comment Templates Reference

### Header Template

```
# Test: [Name]
# Purpose: [One-line description]
# Rules Tested: [Same Day | Bed & Breakfast | Section 104 | Multiple]
# Complexity: [Simple | Complex]
# Key Features: [Comma-separated list]
# Expected Outcome: [Gain/Loss amounts]
#
# Verification Status: [Not Started | Verified | Discrepancy Found]
# Verified By: [Name/Date]
# Verification Notes: [Brief summary]
```

### Inline Comment (Simple)

```
# [Brief description of what happens]
[transaction line]
# [Calculation note if helpful]
```

### Detailed Verification Section

```
# === DETAILED VERIFICATION ===
# Tax Year: YYYY/YYYY
#
# Transaction N: [Date] [Action] [Details]
#   Rule: [Which rule applies]
#   [Relevant calculations]
#   Pool After: [If applicable]
#
# FINAL RESULT:
#   Total Gain: £X
#   Total Loss: £Y
#   Verification: ✓ Matches expected output
# === END DETAILED VERIFICATION ===
```

---

## Tips & Best Practices

### General

1. **Always read TAX_RULES.md first** - Understanding the rules prevents errors
2. **Work in tax year order** - Easier to track pool across years
3. **Show your work** - Future maintainers will thank you
4. **Use consistent notation** - Stick to template formats
5. **Be explicit about rules** - Don't assume reader knows which rule applies

### Calculations

1. **Track expenses separately** - Easy to forget in mental math
2. **Maintain running pool totals** - Catch errors early
3. **Check your arithmetic** - Small errors compound
4. **Use exact decimals** - Don't round until final result
5. **Verify dates carefully** - Off-by-one errors in B&B window are common

### Documentation

1. **Prioritize clarity** - Comments are for humans, not computers
2. **Reference HMRC sections** - Adds authority and helps future lookups
3. **Explain non-obvious decisions** - Why this rule and not another?
4. **Keep it concise** - Don't write essays, just enough to understand
5. **Update status promptly** - Verification status should always be current

### Discrepancies

1. **Don't change tests lightly** - Requires HMRC proof per constitution
2. **Fix code first** - Tests are source of truth
3. **Document everything** - Future you will need this context
4. **Consult HMRC guidance** - Not implementation consensus
5. **Re-run full suite** - Ensure no regressions

---

## Common Pitfalls

### Matching Rule Errors

❌ **Wrong:** Assuming Section 104 when B&B applies

- Check 30-day window carefully!

❌ **Wrong:** Forgetting Same Day takes precedence

- Always check same-day transactions first

✅ **Correct:** Follow hierarchy strictly: Same Day → B&B → Section 104

### Pool Calculation Errors

❌ **Wrong:** Pool cost = Latest purchase price

- Pool uses average cost across all acquisitions!

❌ **Wrong:** Forgetting to update pool after disposal

- Must reduce both quantity and total cost

✅ **Correct:** Average cost = Total pool cost / Total pool quantity

### Expense Handling

❌ **Wrong:** Ignoring expenses in gain calculation

- Both buy and sell expenses reduce gain

❌ **Wrong:** Adding expenses to pool before acquisition

- Expenses added when calculating pool entry cost

✅ **Correct:** Include all allowable expenses per HMRC CG15250

### Tax Year Boundaries

❌ **Wrong:** Using calendar year (Jan 1 - Dec 31)

- UK tax year is April 6 to April 5!

❌ **Wrong:** Forgetting carried losses

- Losses carry forward to future years

✅ **Correct:** Track gains/losses per tax year, carry losses forward

---

## Quick Reference: HMRC Guidance

| Topic            | HMRC Reference | Key Points                                         |
| ---------------- | -------------- | -------------------------------------------------- |
| Same Day Rule    | CG51560        | Acquisitions and disposals same day match first    |
| B&B Rule         | CG51560        | 30 days forward from disposal, chronological order |
| Section 104 Pool | CG51575        | Average cost basis, update with each transaction   |
| Pool Examples    | CG51590        | Official worked examples with calculations         |
| Expenses         | CG15250        | What's allowable (broker fees, stamp duty, etc.)   |
| Stock Splits     | CG51746        | No disposal, adjust quantity and cost              |
| Capital Returns  | CG58620        | Reduce pool cost, may trigger deemed disposal      |
| Tax Years        | CG10220        | 6 April to 5 April annually                        |
| Losses           | CG15750        | Carry forward indefinitely, use efficiently        |

---

## Getting Help

If stuck:

1. **Re-read TAX_RULES.md** - Most questions answered there
2. **Check HMRC CG51590** - Official examples very helpful
3. **Review similar test** - See how other tests handle same scenario
4. **Ask for review** - Get second pair of eyes
5. **Document uncertainty** - Better to note "unclear" than guess

---

**Document Status:** Quickstart guide complete
**Last Updated:** 2025-12-08
