# UK Capital Gains Tax: HMRC Share Matching Rules Research

**Research Date:** 2025-12-08
**Purpose:** Comprehensive reference for validating CGT calculations in cgt-tool

---

## UK CGT Matching Rules

### Overview of Share Identification Hierarchy

When disposing of shares, HMRC requires matching in this strict order:

1. **Same Day Rule** - Match with acquisitions on the same day
2. **Bed & Breakfast Rule (30-day rule)** - Match with acquisitions within 30 days after disposal
3. **Section 104 Pooling** - Match with the pooled holding (average cost basis)

This hierarchy applies to disposals from 6 April 2008 onwards for Capital Gains Tax purposes.

---

## 1. Same Day Rule

### How It Works

**Legal Basis:** TCGA92/S105(1)(b)

If shares are both acquired and disposed of on the same day, the disposal is identified first against the acquisition on the same day.

### Key Principles

- **Aggregation:** All shares of the same class acquired by the same person on the same day and in the same capacity are treated as a single transaction
- **Aggregation of disposals:** All shares of the same class disposed of by the same person on the same day are treated as a single disposal
- **Priority:** This rule takes precedence over all other identification rules
- **Exclusion from pool:** Shares matched under the same day rule do not enter the Section 104 holding

### Treatment of Excess

- **Excess disposals:** If disposals exceed same-day acquisitions, the excess shares are identified using the next rule (30-day rule, then Section 104)
- **Excess acquisitions:** If acquisitions exceed same-day disposals, surplus shares enter the Section 104 holding (unless matched under the 30-day rule)

### Examples

**Example 1: Equal same-day transactions**

- 15 March 2023: Buy 1,000 shares at £5,000
- 15 March 2023: Sell 1,000 shares at £6,000
- **Result:** Gain of £1,000. No shares enter the pool.

**Example 2: Excess same-day acquisitions**

- 10 June 2023: Buy 2,000 shares at £10,000
- 10 June 2023: Sell 1,500 shares at £8,000
- **Result:** 1,500 shares matched at cost of £7,500 (proportional). Remaining 500 shares (£2,500) enter Section 104 holding.

### Edge Cases

- Multiple buy/sell transactions on the same day are aggregated before matching
- Shares acquired under employee share schemes on the same day may be eligible for alternative treatment via election
- The rule applies regardless of the order of transactions within the day

---

## 2. Bed & Breakfast Rule (30-Day Rule)

### How It Works

**Legal Basis:** TCGA92/S106A(5) and (5A)

Disposals are matched with acquisitions made within the following 30 days, taking priority over Section 104 holding identification.

### Requirements (All Must Be Met)

1. **Same class of shares** in the same company
2. **Same person and capacity** (e.g., both personal holdings, not one personal and one in trust)
3. **Acquired within 30 days after the disposal** (not before)

### Key Principles

- **Forward matching only:** Purchases within 30 days AFTER the sale are matched, not before
- **Priority:** Takes priority over Section 104 holding but not over same day rule
- **Purpose:** Normally reduces or eliminates the gain/loss that would arise from matching with existing holdings
- **Anti-avoidance:** Prevents taxpayers from realizing losses while maintaining economic exposure

### Non-Resident Exclusion

The 30-day rule **does not apply** if:

- The person was non-UK resident at the time of the later acquisition
- The acquisition occurred on or after 22 March 2006
- This applies regardless of residence status at disposal

### Reorganisation Exclusions

The rule does not apply if between disposal and acquisition there is:

- A bonus issue or rights issue
- A part disposal under TCGA92/S122-123

### Matching Priority with Multiple Acquisitions

If multiple acquisitions occur within the 30-day window, they are matched:

1. In chronological order (earliest first)
2. On a first-in, first-out (FIFO) basis

### Examples

**Example 1: Simple bed & breakfast (from HMRC CG51560)**

- 1 July 2011: Miss A sells 1,000 shares
- 31 July 2011: Miss A repurchases 1,000 shares (day 30)
- **Result:** The disposal matches with the repurchase, not with any existing Section 104 holding

**Example 2: Partial matching within 30 days (from HMRC CG51560)**

- 27 March 2012: Mr B sells 1,700 shares
- 30 March 2012: Mr B buys 500 shares (day 3)
- **Result:** 500 of the disposed shares match with the purchase. The remaining 1,200 disposed shares match with Section 104 holding. The 500 purchased shares do not enter the Section 104 holding.

**Example 3: Outside 30-day window (from HMRC CG51560)**

- 28 February 2009: Mrs C sells 2,000 shares
- 31 March 2009: Mrs C buys 3,000 shares (day 31)
- **Result:** Outside the 30-day window. Disposal matches with Section 104 holding. All 3,000 purchased shares enter the Section 104 holding.

### Important Notes

- The 30-day period is measured in calendar days
- Days are counted from the day after the disposal
- The purchase on day 30 is still within the window
- If shares are matched under this rule, they cannot also be added to the Section 104 holding

---

## 3. Section 104 Pooling

### How It Works

**Legal Basis:** TCGA92/S104

All shares of the same class in the same company acquired from 1 April 1982 onwards are pooled together in a single "Section 104 holding." These shares are treated as indistinguishable parts of a single asset with an average cost basis.

### Pool Structure

The Section 104 holding maintains:

- **Number of shares** in the pool
- **Pool of allowable expenditure** (total cost including acquisition expenses)

### Average Cost Basis

Each share in the pool is treated as acquired at the same average cost:

```
Average cost per share = Total pool cost ÷ Number of shares in pool
```

### How Transactions Affect the Pool

#### Acquisitions

When shares are purchased (and not matched under same day or 30-day rules):

- Add the number of shares to the pool
- Add the cost (purchase price + allowable expenses) to the pool expenditure

#### Disposals

When shares are sold from the pool:

- Reduce the number of shares in the pool
- Calculate allowable cost as a fraction of pool expenditure:

```
Allowable cost = (Shares sold ÷ Shares in pool before disposal) × Total pool cost
```

- Reduce pool expenditure by the allowable cost
- Calculate gain/loss: Proceeds - Allowable cost

#### Complete Disposal

If all shares are disposed of:

- The allowable expenditure is the entire pool cost
- The pool is eliminated

### Allowable Expenditure in Pool

The pool includes all allowable expenditure under TCGA92/S38(1)(a) and (b):

- Purchase price of shares
- Incidental costs of acquisition (broker fees, stamp duty, etc.)
- Incidental costs of disposal (when shares are sold)

### Section 104 Holding Examples (from HMRC CG51590)

#### Example 1: Ms Davy's Share Disposal

**Acquisitions:**

- 15 April 2006: 1,000 shares at £1,300
- 4 August 2006: 1,000 shares at £1,450
- 19 January 2007: 500 shares at £950

**Section 104 Pool (before disposal):**

- Total shares: 2,500
- Total cost: £3,700
- Average cost per share: £1.48

**Disposal on 10 December 2010:**

- Sold 2,200 shares for £7,700
- Apportioned cost: (2,200 ÷ 2,500) × £3,700 = £3,256
- **Chargeable gain:** £7,700 - £3,256 = £4,444

**Remaining pool:**

- Shares: 300
- Cost: £444
- Average cost per share: £1.48 (unchanged)

---

#### Example 2: Mr Browne's Rights Issue

**Acquisitions:**

- 17 August 2008: 10,000 shares at £2,500
- 1 April 2009: 10,000 shares at £2,600
- 8 October 2009: Rights issue—4,000 shares at £1,060

**Section 104 Pool:**

- Total shares: 24,000
- Total cost: £6,160
- Average cost per share: £0.257

**Disposal on 10 December 2012:**

- Sold 7,500 shares for £3,000
- Apportioned cost: (7,500 ÷ 24,000) × £6,160 = £1,925
- **Chargeable gain:** £3,000 - £1,925 = £1,075

**Remaining pool:**

- Shares: 16,500
- Cost: £4,236
- Average cost per share: £0.257 (unchanged)

**Note:** Rights issues are treated as share reorganisations under TCGA92/S127. The rights issue shares and their cost are simply added to the Section 104 holding.

---

#### Example 3: Mrs Mountain's Long-Term Holding

**Section 104 Pool (consolidated from 1979-2005):**

- Shares: 21,500
- Total cost: £83,500
- Average cost per share: £3.88

**Disposal on 13 June 2013:**

- Sold 16,500 shares for £114,675
- Apportioned cost: (16,500 ÷ 21,500) × £83,500 = £64,081
- **Chargeable gain:** £114,675 - £64,081 = £50,594

**Remaining pool:**

- Shares: 5,000
- Cost: £19,419
- Average cost per share: £3.88 (unchanged)

---

#### Example 4: Peninsula Trust's Multiple Acquisitions

**Section 104 Pool (September 1997 through November 2005):**

- Shares: 45,000
- Total cost: £33,600
- Average cost per share: £0.747

**Disposal on 23 February 2010:**

- Sold 20,000 shares for £39,000
- Apportioned cost: (20,000 ÷ 45,000) × £33,600 = £14,934
- **Chargeable gain:** £39,000 - £14,934 = £24,066

**Remaining pool:**

- Shares: 25,000
- Cost: £18,666
- Average cost per share: £0.747 (unchanged)

### Important Notes

- The average cost per share remains constant within the pool until new shares are added
- Disposals reduce both the number of shares and the pool cost proportionally
- The pool operates continuously across tax years
- Shares matched under same day or 30-day rules never enter the pool

---

## Tax Year Boundaries

### UK Tax Year Definition

**Tax year runs from 6 April to 5 April** of the following year.

Examples:

- Tax year 2024/25: 6 April 2024 to 5 April 2025
- Tax year 2023/24: 6 April 2023 to 5 April 2024

### Impact on CGT Calculations

- **Annual exempt amount:** Applied per tax year (£12,300 for 2022/23, £6,000 for 2023/24, £3,000 for 2024/25)
- **Gains and losses:** Calculated within each tax year
- **Loss carryforward:** Unused losses carried forward to future tax years
- **Share matching:** Same day and 30-day rules can span tax year boundaries
- **Section 104 holding:** Maintains continuity across tax years

### Example: Matching Across Tax Year Boundary

- 28 March 2024: Sell 1,000 shares (tax year 2023/24)
- 10 April 2024: Buy 1,000 shares (tax year 2024/25, day 13 after sale)
- **Result:** Bed & breakfast rule applies (within 30 days). The sale in 2023/24 matches with the purchase in 2024/25.

---

## Special Cases

### 1. Stock Splits and Share Reorganisations

**Legal Basis:** TCGA92/S127-S128

#### Bonus Issues (Free Shares)

- **Treatment:** Not a disposal or acquisition for CGT purposes
- **Effect on pool:** Add bonus shares to Section 104 holding at zero cost
- **Cost basis:** Pool cost remains unchanged; average cost per share decreases

**Example:**

- Pool: 1,000 shares at £5,000 (£5 per share)
- 1-for-1 bonus issue: Receive 1,000 free shares
- New pool: 2,000 shares at £5,000 (£2.50 per share)

#### Rights Issues (Paid Shares)

- **Treatment:** Not a disposal; shares and cost added to Section 104 holding
- **Effect on pool:** Add both shares and payment to the pool under TCGA92/S128
- **Cost basis:** Pool cost includes rights payment

**Example:**

- Pool: 10,000 shares at £2,500
- 2-for-5 rights issue at £0.25: Buy 4,000 shares for £1,000
- New pool: 14,000 shares at £3,500

#### Stock Splits

- **Treatment:** Similar to bonus issue - reorganisation, not disposal
- **Effect on pool:** Increase share count proportionally, pool cost unchanged
- **Cost basis:** Average cost per share decreases proportionally

**Example:**

- Pool: 500 shares at £10,000 (£20 per share)
- 2-for-1 split: Now 1,000 shares
- New pool: 1,000 shares at £10,000 (£10 per share)

### 2. Capital Returns and Distributions

**Legal Basis:** TCGA92/S122(1)

#### What Is a Capital Distribution?

Any distribution from a company in money or money's worth, except distributions that constitute income for Income Tax purposes.

#### CGT Treatment

- **Deemed disposal:** Treated as disposing of an interest in the shares
- **Not actual disposal:** Shares remain in holding, but pool cost is reduced
- **Section 104 effect:** Reduce pool expenditure by the capital return amount

#### Small Capital Distributions

If the capital distribution is small:

- May not trigger immediate disposal
- Reduces pool cost for future gain calculations

### 3. Allowable Expenses

**Legal Basis:** TCGA92/S38(1) and TCGA92/S38(2)

#### Allowable Incidental Costs

**Acquisition costs:**

- Broker fees and commissions
- Stamp Duty and Stamp Duty Reserve Tax (SDRT)
- Legal fees for transfer/conveyance
- Valuation costs (if required for CGT computation)

**Disposal costs:**

- Broker fees and commissions
- Advertising costs to find a buyer
- Legal fees for transfer

#### Must Be "Wholly and Exclusively"

Expenditure must be incurred wholly and exclusively for the purpose of acquiring or disposing of the shares.

#### NOT Allowable

- General investment advice or portfolio management fees
- Subscriptions to financial periodicals
- Accountancy fees for computing tax liability
- Costs of general market research

#### Important for Quoted Securities

For publicly traded shares, allowable accountancy fees are typically minimal since market value is readily ascertainable.

### 4. Capital Losses and Carryforward

**Legal Basis:** TCGA92/S16(2A)

#### How Losses Are Used

Losses must be applied in this order:

1. **Current year losses:** Deduct from current year gains
2. **Current year net gains vs annual exempt amount:**
   - If net gains ≤ annual exempt amount: Carry forward all unused losses
   - If net gains > annual exempt amount: Use only enough losses to reduce gains to the annual exempt amount
3. **Remaining losses:** Carried forward indefinitely to future tax years

#### Loss Notification Requirements

- Losses must be notified to HMRC in a quantified amount to be allowable
- Notification must occur within 4 years of the end of the tax year of disposal
- Losses not notified within the time limit are not allowable

#### Loss Carryforward Example

**Tax Year 2023/24:**

- Gains: £20,000
- Current year losses: £5,000
- Net gains: £15,000
- Annual exempt amount: £6,000
- Brought forward losses: £10,000

**Calculation:**

- Net gains (£15,000) > Annual exempt amount (£6,000)
- Need to reduce net gains by: £15,000 - £6,000 = £9,000
- Use £9,000 of brought forward losses
- Taxable gains: £6,000 (the annual exempt amount)
- Remaining losses carried forward: £10,000 - £9,000 = £1,000

#### Special Restrictions

- Losses from disposals to connected persons (e.g., spouse, family) can generally only be set against gains from the same person
- Losses must be computed in the same way as gains (TCGA92/S16(1) and (2))

### 5. Pre-2008 Holdings

#### Transition Rules

From 6 April 2008, the share identification rules changed. Pre-2008 holdings were converted into the new Section 104 holding system.

#### 1982 Valuation

For shares held on 31 March 1982:

- Taxpayers may elect to use market value on 31 March 1982 as the cost base
- This can avoid gains that accrued before the introduction of CGT indexation
- External valuation may be required for unquoted shares

---

## Complete Calculation Example

### Scenario: Multiple Transactions Spanning Matching Rules

**Holding as of 1 January 2024:**

- Section 104 pool: 5,000 shares at £20,000 (£4 per share)

**Transactions in 2024:**

1. **15 March 2024:** Buy 1,000 shares at £5,500 (including £50 expenses)
2. **15 March 2024:** Sell 800 shares at £5,200
3. **20 March 2024:** Sell 1,500 shares at £7,200
4. **25 March 2024:** Buy 500 shares at £2,600 (including £25 expenses)
5. **30 April 2024:** Sell 2,000 shares at £10,000

### Step-by-Step Calculation

#### Transaction 1-2: Same Day Rule (15 March 2024)

- Buy 1,000 shares at £5,500
- Sell 800 shares at £5,200
- **Same day matching:** 800 shares matched
  - Cost: (800 ÷ 1,000) × £5,500 = £4,400
  - Gain: £5,200 - £4,400 = £800
- **Remaining purchases:** 200 shares at £1,100 added to Section 104 pool

**Updated Section 104 pool:**

- Shares: 5,000 + 200 = 5,200
- Cost: £20,000 + £1,100 = £21,100

#### Transaction 3-4: 30-Day Rule (20 & 25 March 2024)

- 20 March: Sell 1,500 shares
- 25 March: Buy 500 shares (day 5 after sale)
- **Bed & breakfast matching:** 500 shares matched
  - Cost: £2,600
  - Proceeds: (500 ÷ 1,500) × £7,200 = £2,400
  - **Loss:** £2,400 - £2,600 = -£200
- **Remaining sale:** 1,000 shares matched with Section 104 pool
  - Cost: (1,000 ÷ 5,200) × £21,100 = £4,058
  - Proceeds: (1,000 ÷ 1,500) × £7,200 = £4,800
  - Gain: £4,800 - £4,058 = £742

**Updated Section 104 pool:**

- Shares: 5,200 - 1,000 = 4,200
- Cost: £21,100 - £4,058 = £17,042

#### Transaction 5: Section 104 Pool (30 April 2024)

- 30 April: Sell 2,000 shares at £10,000
- No same day or 30-day matching available
- **Section 104 matching:**
  - Cost: (2,000 ÷ 4,200) × £17,042 = £8,115
  - Gain: £10,000 - £8,115 = £1,885

**Final Section 104 pool:**

- Shares: 4,200 - 2,000 = 2,200
- Cost: £17,042 - £8,115 = £8,927

### Summary for Tax Year 2024/25

- Same day gain: £800
- Bed & breakfast loss: -£200
- Section 104 gain (20 March): £742
- Section 104 gain (30 April): £1,885
- **Total net gains:** £3,227
- Annual exempt amount (2024/25): £3,000
- **Taxable gains:** £227

---

## HMRC References

### Primary Sources

1. [CG51500 - Share Identification Rules: Introduction](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51500)
2. [CG51550 - Share Identification Rules for CGT from 6.4.2008: Outline](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51550)
3. [CG51560 - Same Day and Bed & Breakfast Identification Rules](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560)
4. [CG51575 - The Section 104 Holding in Detail](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51575)
5. [CG51590 - Share Identification Rules: Examples](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51590)

### Share Reorganisations and Special Cases

6. [HS284 - Shares and Capital Gains Tax (2021)](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet/hs284-shares-and-capital-gains-tax-2021)
7. [HS285 - Share Reorganisations, Company Takeovers and Capital Gains Tax (2021)](https://www.gov.uk/government/publications/share-reorganisations-company-takeovers-and-capital-gains-tax-hs285-self-assessment-helpsheet/hs285-share-reorganisations-company-takeovers-and-capital-gains-tax-2021)
8. [CG51746 - Reorganisations: Bonus and Rights Issues](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51746)
9. [CG51620 - Share Identification Rules for Corporation Tax: Section 104 Holding](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51620)

### Expenses and Losses

10. [CG15250 - Expenditure: Incidental Costs of Acquisition and Disposal](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg15250)
11. [CG15260 - Incidental Costs: Specific Examples](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg15260)
12. [Capital Gains Tax: Losses](https://www.gov.uk/capital-gains-tax/losses)
13. [CG21520 - Individuals: Losses: Relief for Losses: Examples 1 to 5](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg21520)

### Capital Returns

14. [CG58620 - Company Purchases Own Shares: Repayment/Redemption Share Capital](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg58620)
15. [CG58650 - Company Purchases Own Shares: Capital Treatment: CGT Liability](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg58650)

### Tax Year and Reporting

16. [Capital Gains Tax: What You Pay It On, Rates and Allowances](https://www.gov.uk/capital-gains-tax/work-out-need-to-pay)
17. [Tax When You Sell Shares: Work Out Your Gain](https://www.gov.uk/tax-sell-shares/work-out-your-gain)

---

## Key Takeaways for Test Validation

01. **Matching order is strict:** Same day → 30-day → Section 104
02. **Same day rule:** Always applies first, aggregates all same-day transactions
03. **30-day rule:** Forward-looking only (purchases after sale), measured in calendar days
04. **Section 104:** Maintains continuous average cost pool across all tax years
05. **Proportional cost allocation:** Use (shares disposed ÷ total shares) × total cost
06. **Tax year boundaries:** 6 April to 5 April, but matching rules can span years
07. **Expenses:** Include acquisition and disposal costs in pool (if wholly and exclusively incurred)
08. **Losses:** Must be notified within 4 years; used efficiently to preserve annual exempt amount
09. **Reorganisations:** Bonus/rights issues add shares to pool without triggering disposal
10. **Capital returns:** Reduce pool cost; may trigger deemed disposal

---

## cgtcalc Repository Comparison

**Repository:** https://github.com/mattjgalloway/cgtcalc
**Commit:** 896d91486805e27fcea0e851ee01868b86e161f5
**Download Date:** 2025-12-08
**Commit Date:** 2025-11-21 21:42:29 +0000

### Test Location

- **cgtcalc:** `Tests/CGTCalcCoreTests/TestData/Examples/Inputs/*.txt`
- **cgt-tool:** `tests/data/*.cgt`

### Test Count

- **cgtcalc:** 21 test input files
- **cgt-tool:** 22 test files (.cgt) + 3 additional (unsorted_transactions, plus note on file count difference)

### DSL Syntax Differences

**cgtcalc format:**

```
SELL 28/08/2018 GB00B41YBW71 10 4.6702 12.5
BUY 28/08/2018 GB00B41YBW71 10 4.1565 12.5
```

**cgt-tool format:**

```
2018-08-28 BUY GB00B41YBW71 10 @ 4.1565 EXPENSES 12.5
2018-08-28 SELL GB00B41YBW71 10 @ 4.6702 EXPENSES 12.5
```

**Key Differences:**

1. **Date format:** cgtcalc uses DD/MM/YYYY, cgt-tool uses YYYY-MM-DD (ISO 8601)
2. **Transaction order:** cgtcalc uses ACTION first, cgt-tool uses DATE first
3. **Price separator:** cgtcalc uses space, cgt-tool uses `@` symbol
4. **Expenses keyword:** cgtcalc uses positional (last number), cgt-tool uses explicit `EXPENSES` keyword
5. **Additional syntax:** cgt-tool supports `TAX`, `RATIO`, `DIVIDEND`, `CAPRETURN` keywords

### Line Order Differences

**cgtcalc:** Appears to use reverse chronological order (latest transactions first)
**cgt-tool:** Our clarification noted we use "reversed line order" - needs verification during comparison

### Test Mapping

| cgtcalc Test Name                   | cgt-tool Test Name                  | Match Status     |
| ----------------------------------- | ----------------------------------- | ---------------- |
| 2024_2025_SpecialYear.txt           | 2024_2025_SpecialYear.cgt           | ✅ Exact match   |
| AssetEventsNotFullSale.txt          | AssetEventsNotFullSale.cgt          | ✅ Exact match   |
| AssetEventsNotFullSale2.txt         | AssetEventsNotFullSale2.cgt         | ✅ Exact match   |
| Blank.txt                           | Blank.cgt                           | ✅ Exact match   |
| BuySellAllBuyAgainCapitalReturn.txt | BuySellAllBuyAgainCapitalReturn.cgt | ✅ Exact match   |
| CarryLoss.txt                       | CarryLoss.cgt                       | ✅ Exact match   |
| GainsAndLosses.txt                  | GainsAndLosses.cgt                  | ✅ Exact match   |
| HMRCExample1.txt                    | HMRCExample1.cgt                    | ✅ Exact match   |
| MultipleMatches.txt                 | MultipleMatches.cgt                 | ✅ Exact match   |
| SameDayMerge.txt                    | SameDayMerge.cgt                    | ✅ Exact match   |
| SameDayMergeInterleaved.txt         | SameDayMergeInterleaved.cgt         | ✅ Exact match   |
| Simple.txt                          | Simple.cgt                          | ✅ Exact match   |
| SimpleTwoSameDay.txt                | SimpleTwoSameDay.cgt                | ✅ Exact match   |
| WithAssetEvents.txt                 | WithAssetEvents.cgt                 | ✅ Exact match   |
| WithAssetEventsBB.txt               | WithAssetEventsBB.cgt               | ✅ Exact match   |
| WithAssetEventsMultipleYears.txt    | WithAssetEventsMultipleYears.cgt    | ✅ Exact match   |
| WithAssetEventsSameDay.txt          | WithAssetEventsSameDay.cgt          | ✅ Exact match   |
| WithSplitBB.txt                     | WithSplitBB.cgt                     | ✅ Exact match   |
| WithSplitS104.txt                   | WithSplitS104.cgt                   | ✅ Exact match   |
| WithUnsplitBB.txt                   | WithUnsplitBB.cgt                   | ✅ Exact match   |
| WithUnsplitS104.txt                 | WithUnsplitS104.cgt                 | ✅ Exact match   |
| (none)                              | unsorted_transactions.cgt           | ⚠️ cgt-tool only |

### Test Coverage Analysis

**Perfect Match:** 21 of 21 cgtcalc tests have equivalent cgt-tool tests
**Additional Tests:** cgt-tool has 1 additional test (unsorted_transactions.cgt) not in cgtcalc

### Gaps Identified

**In cgtcalc but not in cgt-tool:** None
**In cgt-tool but not in cgtcalc:**

- `unsorted_transactions.cgt` - Tests parser handling of unsorted transaction order

### Next Steps

1. **Transaction-level comparison:** Compare actual transaction data (accounting for DSL syntax differences)
2. **Line order verification:** Confirm whether line order is reversed and adjust comparison accordingly
3. **Expected output comparison:** Compare .json outputs with cgtcalc .txt outputs (accounting for format differences)
4. **Discrepancy investigation:** Document any differences in calculated results

---

## Verification Workflow Design

### Comment Templates

#### Header Comment Template for .cgt Files

```
# Test: [Test Name]
# Purpose: [What this test validates]
# Rules Tested: [Same Day | Bed & Breakfast | Section 104 | Multiple]
# Complexity: [Simple | Complex]
# Key Features: [e.g., "Multi-year", "Stock splits", "Capital returns"]
# Expected Outcome: [Brief description of expected gain/loss]
#
# Verification Status: [Not Started | In Progress | Verified | Discrepancy Found]
# Verified By: [Name/Date or "AUTOMATED"]
# Verification Notes: [Brief summary or "See inline comments"]
```

#### Inline Verification Comment Template

```
# [Transaction description]
# Rule Applied: [Same Day | B&B | Section 104]
# Calculation: [Brief calculation note]
# Pool Status After: [If applicable: quantity X shares @ avg cost Y]
```

#### Detailed Step-by-Step Template (for 5+ representative cases)

```
# === DETAILED VERIFICATION ===
# Tax Year: YYYY/YYYY
#
# Transaction 1: [Date] [Action] [Details]
#   Rule: [Which matching rule applies]
#   Cost Basis: [Calculation]
#   Pool Adjustment: [If applicable]
#
# Transaction 2: [Date] [Action] [Details]
#   Rule: [Which matching rule applies]
#   Matched Against: [Reference to earlier transaction]
#   Calculation: Proceeds £X - Cost £Y = Gain/Loss £Z
#   Pool Status: [Quantity @ avg cost]
#
# [Continue for all transactions]
#
# FINAL RESULT:
#   Total Gain: £[Amount]
#   Total Loss: £[Amount]
#   Verification: ✓ Matches expected output
# === END DETAILED VERIFICATION ===
```

### Discrepancy Resolution Workflow

1. **Document Discrepancy**

   - Our calculated result
   - Expected result (from .json or cgtcalc)
   - Magnitude of difference

2. **Consult HMRC Guidance**

   - Review relevant sections from TAX_RULES.md
   - Check specific HMRC manual sections (CG51500-CG51600)
   - Verify against worked examples in HMRC documentation

3. **Determine Root Cause**

   - **Our code bug:** Incorrect matching logic, pool calculation error, etc.
   - **Our test bug:** Incorrect expected values in .json file
   - **cgtcalc bug:** cgtcalc may have incorrect interpretation
   - **Legitimate difference:** Different handling of edge case, both valid

4. **Resolution Action**

   - **Code bug:** Fix calculator code, verify against HMRC rules, re-run all tests
   - **Test bug:** Update .json with proof from HMRC guidance, document justification
   - **cgtcalc difference:** Document rationale, verify our approach is HMRC-compliant
   - **Legitimate difference:** Document both approaches, choose HMRC-endorsed method

5. **Document Resolution**

   - Add comment to test file explaining resolution
   - Update verification status
   - Note HMRC reference used for decision

### Priority Tests for Detailed Verification

Based on complexity and rule diversity, these tests should receive detailed step-by-step verification (per SC-009):

1. **HMRCExample1.cgt** - Official HMRC example, authoritative
2. **WithAssetEventsBB.cgt** - B&B with asset events, multi-year
3. **WithAssetEventsMultipleYears.cgt** - Most complex, 10 transactions, multi-year
4. **MultipleMatches.cgt** - Demonstrates all three rules
5. **SameDayMerge.cgt** - Core Same Day logic with merging
6. **CarryLoss.cgt** - Loss carryover across years

### Verification Checklist

For each test file:

- [ ] Read test file transactions
- [ ] Add header comment with test metadata
- [ ] Identify matching rules to be applied
- [ ] Manually calculate expected result (simple verification)
- [ ] If priority test: Add detailed step-by-step calculations
- [ ] Compare with .json expected output
- [ ] If discrepancy: Follow resolution workflow
- [ ] Mark verification status complete
- [ ] Run `cargo test` to confirm test still passes

---

---

## Test Coverage Analysis (Completed 2025-12-08)

### Coverage Summary

**Total test files:** 22 (.cgt files in tests/data/)

**Coverage by rule:**

- Same Day Rule: 10 tests (Simple, SimpleTwoSameDay, SameDayMerge, SameDayMergeInterleaved, GainsAndLosses, MultipleMatches, CarryLoss, 2024_2025_SpecialYear, WithAssetEventsSameDay)
- Bed & Breakfast Rule: 5 tests (MultipleMatches, WithSplitBB, WithUnsplitBB, WithAssetEventsBB)
- Section 104 Pooling: 12 tests (HMRCExample1, WithSplitS104, WithUnsplitS104, WithAssetEvents, WithAssetEventsMultipleYears, AssetEventsNotFullSale, AssetEventsNotFullSale2, BuySellAllBuyAgainCapitalReturn, unsorted_transactions)
- Stock Splits/Consolidations: 4 tests (WithSplitS104, WithUnsplitS104, WithSplitBB, WithUnsplitBB)
- Capital Returns: 6 tests (WithAssetEvents, WithAssetEventsBB, WithAssetEventsMultipleYears, AssetEventsNotFullSale, AssetEventsNotFullSale2, BuySellAllBuyAgainCapitalReturn)
- Dividends: 4 tests (WithAssetEvents, WithAssetEventsBB, WithAssetEventsMultipleYears, AssetEventsNotFullSale/2)

**Coverage by complexity:**

- Simple: 7 tests
- Medium: 9 tests
- Complex: 6 tests (all 6 priority tests with detailed verification)

**cgt-tool exclusive tests:**

- `unsorted_transactions.cgt` - Tests parser handling of non-chronological input order

### Gaps Analysis

**No critical gaps identified.** The test suite covers:

- All three matching rules (Same Day, B&B, Section 104)
- Multi-year scenarios
- Stock splits and consolidations
- Capital returns and dividends
- Gains and losses
- Tax year boundaries
- Edge cases (empty input, unsorted input)

**Potential future additions (not required for current scope):**

- Multiple tickers with interacting B&B rules
- Very large pool sizes for precision testing
- Negative pool cost scenarios (edge case)
- Pre-2008 holdings (historical migration scenarios)

### Verification Status

All 22 test files have been manually verified with calculations documented:

- 6 priority tests: Detailed step-by-step verification
- 16 simple/medium tests: Header comments with verification notes
- 100% coverage of test cases

---

**Document Status:** Research and verification complete
**Last Updated:** 2025-12-08
