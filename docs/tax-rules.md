# UK Capital Gains Tax: Share Matching Rules

This document explains the UK Capital Gains Tax (CGT) share identification and matching rules used by this calculator. These rules determine how disposals (sales) of shares are matched with acquisitions (purchases) to calculate gains and losses.

## Overview

When you sell shares, HMRC requires you to match them with shares you acquired in a specific order. The matching hierarchy is:

1. **Same Day Rule** - Match with shares bought on the same day
2. **Bed & Breakfast Rule (30-day)** - Match with shares bought within the next 30 days
3. **Section 104 Pool** - Match with your pooled holding at average cost

This hierarchy applies to disposals from 6 April 2008 onwards.

---

## Rule 1: Same Day Matching

**HMRC Reference:** CG51560, TCGA92/S105(1)

### Definition

If you buy and sell shares of the same class in the same company on the same day, the sale is matched first against shares bought on that day. This is mandated by TCGA92/S105(1)(a) which requires that disposals are identified with acquisitions on the same day before any other matching rules apply.

### Key Points

- All purchases on the same day are aggregated (treated as one transaction)
- All sales on the same day are aggregated (treated as one disposal)
- Allowable cost for same-day matching uses a weighted average across all same-day purchases (including expenses)
- The rule applies regardless of transaction order within the day
- Matched shares do NOT enter the Section 104 pool
- Expenses are apportioned proportionally

### Example 1: Basic Same Day Match

**Transactions on 15 August 2023:**

- Buy 100 shares at £10.00 each (cost £1,000 + £10 expenses = £1,010)
- Sell 100 shares at £12.00 each (proceeds £1,200 - £10 expenses = £1,190)

**Calculation:**

- Proceeds: £1,190
- Cost: £1,010
- **Gain: £180**

No shares enter the Section 104 pool.

### Example 2: Partial Same Day Match

**Transactions on 20 September 2023:**

- Buy 200 shares at £5.00 each (cost £1,000 + £20 expenses = £1,020)
- Sell 150 shares at £6.00 each (proceeds £900 - £15 expenses = £885)

**Calculation:**

- 150 shares matched against same-day purchase
- Cost for 150 shares: £1,020 × (150/200) = £765
- Proceeds: £885
- **Gain: £120**

Remaining 50 shares (cost £255) enter Section 104 pool.

### Example 3: Same Day Match with Asset Events

When a same-day buy and sell occurs with an existing pool holding, the Same Day rule matches the sale against the same-day acquisition first. This is important for calculating capital return apportionment on remaining holdings.

**Starting Position:** Pool of 40 shares with total cost £7,338.70

**Transactions:**

- 31 May 2023: Capital return of £149.75 (reduces pool cost)
- 5 November 2023: Buy 20 shares at £194.22 (cost £3,888.40)
- 5 November 2023: Sell 40 shares at £194.22 (proceeds £7,768.80)

**Matching:**

1. **Same Day**: 20 shares matched against 5 November purchase (cost £3,888.40)
2. **Section 104**: 20 shares matched from pool (cost £3,677.50 after capital return adjustment)

**Result:**

- Same Day match: Small loss (proceeds slightly less than cost after fees)
- Section 104 match: Gain (original pool cost was lower)
- 20 shares remain in pool

---

## Rule 2: Bed & Breakfast (30-Day Rule)

**HMRC Reference:** CG51560

### Definition

If you sell shares and buy the same type of shares within the following 30 days, the sale is matched with those later purchases. This prevents "bed and breakfasting" - selling shares to crystallize a loss and immediately buying them back.

### Key Points

- Only applies to purchases made AFTER the sale (not before)
- 30-day window starts the day after the sale
- Multiple purchases within 30 days are matched chronologically (earliest first)
- Takes priority over Section 104 pool, but not over Same Day rule
- Non-UK residents at time of purchase may be excluded

### Same Day Priority Over B&B

**HMRC Reference:** TCGA92/S106A(9), S105(1)

When multiple disposals compete for the same acquisition, Same Day matching has absolute priority over B&B matching from earlier disposals. Per TCGA92/S106A(9), the B&B identification rules are "subject to subsection (1) of section 105" (the Same Day rule).

**Example: Competing Claims**

- 1 February: Sell 100 shares (disposal D1, could B&B to 2 February)
- 2 February: Buy 80 shares, Sell 50 shares (disposal D2, Same Day match)

Without priority rules, D1 could consume all 80 shares via B&B, leaving D2 with nothing for Same Day.

**Correct Treatment:**

D2's Same Day claim (50 shares) is reserved first. D1's B&B can only use the remaining 30 shares (80 - 50). D1's remaining 70 shares match against the Section 104 pool.

This ensures Same Day matching is always fully satisfied before B&B from earlier disposals can consume shares.

### Example 1: Basic Bed & Breakfast

**Transactions:**

- 10 March 2023: Sell 100 shares at £8.00 each (proceeds £800 - £12 expenses = £788)
- 25 March 2023: Buy 100 shares at £7.50 each (cost £750 + £12 expenses = £762)

**Calculation:**

- Sale matched with purchase within 30 days
- Proceeds: £788
- Cost: £762 (the later purchase cost)
- **Gain: £26**

Note: If the 25 March purchase had been made on 11 April (32 days later), the sale would have matched against the Section 104 pool instead.

### Example 2: B&B with Section 104 Interaction

**Starting Position:** Section 104 pool of 500 shares at average cost £4.00 (total £2,000)

**Transactions:**

- 1 June 2023: Sell 200 shares at £6.00 each (proceeds £1,200 - £20 expenses = £1,180)
- 15 June 2023: Buy 100 shares at £5.50 each (cost £550 + £10 expenses = £560)

**Calculation:**

- 100 shares matched with B&B purchase (15 June)

  - Proceeds: £1,180 × (100/200) = £590
  - Cost: £560
  - **Gain: £30**

- 100 shares matched from Section 104 pool

  - Proceeds: £1,180 × (100/200) = £590
  - Cost: 100 × £4.00 = £400
  - **Gain: £190**

- **Total Gain: £220**

Pool after: 400 shares (500 - 100 from S104 match) at average cost £4.00

---

## Rule 3: Section 104 Pooling

**HMRC Reference:** CG51575

### Definition

All shares of the same class in the same company are pooled together. When shares are not matched by Same Day or B&B rules, they are matched from this pool at the average cost per share.

### Key Points

- Pool maintains running totals: number of shares and total cost
- Average cost = Total pool cost / Total pool shares
- New purchases add to pool quantity and cost
- Sales remove from pool at average cost
- Pool cost includes allowable expenses (broker fees, stamp duty)
- Pool continues indefinitely across tax years

### Average Cost Calculation

```
Average Cost Per Share = Total Pool Cost / Total Pool Shares

Cost of Disposal = Number of Shares Sold × Average Cost Per Share
```

### Example 1: Simple Pool Calculation

**Transactions:**

- 1 January 2022: Buy 100 shares at £10.00 (cost £1,000 + £15 expenses = £1,015)
- 1 June 2022: Buy 50 shares at £12.00 (cost £600 + £10 expenses = £610)
- 1 December 2022: Sell 80 shares at £15.00 (proceeds £1,200 - £12 expenses = £1,188)

**Pool after purchases:**

- Shares: 100 + 50 = 150
- Cost: £1,015 + £610 = £1,625
- Average cost: £1,625 / 150 = £10.833

**Disposal calculation:**

- Proceeds: £1,188
- Cost: 80 × £10.833 = £866.67
- **Gain: £321.33**

**Pool after sale:**

- Shares: 150 - 80 = 70
- Cost: £1,625 - £866.67 = £758.33
- Average cost remains: £10.833

### Example 2: Multi-Year Pool

**Year 1 (2021/22):**

- 1 May 2021: Buy 200 shares at £5.00 (cost £1,020 including expenses)
- Pool: 200 shares, £1,020, avg £5.10

**Year 2 (2022/23):**

- 1 August 2022: Buy 100 shares at £6.00 (cost £615 including expenses)

- Pool: 300 shares, £1,635, avg £5.45

- 1 February 2023: Sell 150 shares at £7.00 (proceeds £1,035 after expenses)

- Cost from pool: 150 × £5.45 = £817.50

- **Gain: £217.50**

- Pool after: 150 shares, £817.50, avg £5.45

**Year 3 (2023/24):**

- Pool carries forward: 150 shares, £817.50, avg £5.45

---

## Special Cases

### RSU Acquisition Date

**HMRC Reference:** CG14250, ERSM20192

When Restricted Stock Units (RSUs) vest, two dates are relevant:

1. **Vest Date (Lapse Date)**: When vesting conditions are satisfied and the employee becomes unconditionally entitled to the shares
2. **Settlement Date**: When shares are deposited into the brokerage account (typically T+2, i.e., 2 business days after vest)

For CGT purposes, the **vest date** is the acquisition date, not the settlement date. HMRC guidance is clear:

- **CG14250**: "If the contract is conditional the date of disposal is the date all of the conditions are satisfied"
- **ERSM20192**: "An RSU award will vest when all the conditions laid down to be satisfied before the stock or shares may be issued have been met"

For RSU acquisitions, the allowable cost is the **market value at vest** (the value used for income tax at vest). The awards data should therefore supply the vest-date FMV (e.g., `VestFairMarketValue`) rather than a settlement-date value.

Awards exports may also include non-vesting cash actions (e.g., wire transfers or tax withholdings) that do not provide FMV details. These entries do not affect CGT calculations and are ignored for vesting FMV lookups.

This distinction matters for share matching rules:

- **Same Day Rule**: Uses the vest date for matching
- **Bed & Breakfast Rule**: The 30-day window is calculated from the vest date

### Example: RSU Vest Date Impact on Same Day Matching

**Scenario:**

- RSU vests on 15 January 2024 (vest date)
- Shares settle in brokerage on 17 January 2024 (settlement date, T+2)
- Employee sells 30 shares on 15 January 2024

**Correct Treatment (using vest date):**

- Acquisition date: 15 January 2024
- Sale date: 15 January 2024
- **Same Day Rule applies** - sale matches with acquisition

**Incorrect Treatment (using settlement date):**

- Acquisition date: 17 January 2024
- Sale date: 15 January 2024
- Same Day Rule would NOT apply (dates differ)
- Sale would incorrectly match against Section 104 pool

The cgt-tool uses the vest date from the awards file when processing RSU vesting transactions from broker exports.

### Stock Splits and Bonus Issues

**HMRC Reference:** CG51746

When a company splits its shares or issues bonus shares:

- The number of shares in your pool increases
- The total cost remains the same
- The average cost per share decreases proportionally

**Example:** 2-for-1 split on pool of 100 shares at £1,000:

- Before: 100 shares, £1,000 total, £10.00 avg
- After: 200 shares, £1,000 total, £5.00 avg

### Capital Returns

**HMRC Reference:** CG58620

When a company returns capital to shareholders:

- Reduces the cost base of your holding (TCGA92/S122(2), CG57844)
- The reduction is apportioned across the Section 104 pool based on shares held
- This treatment applies only when the distribution is "small" per S122(2)

**Example 1: Simple Capital Return**

£2.00 per share capital return on 100 shares (pool cost £800):

- Return: 100 × £2.00 = £200
- New pool cost: £800 - £200 = £600

### Capital Return Exceeding Allowable Cost

Per CG57847, if the capital distribution exceeds allowable expenditure,
TCGA92/S122(2) does not apply. The taxpayer must use either:

- **S122(1)**: Part-disposal treatment (full computation)
- **S122(4)**: Election to reduce distribution by remaining allowable cost

This tool does not currently support either treatment. If a `CAPRETURN`
would reduce pool cost below zero, calculation fails with an error
referencing S122 and CG57847.

**Example 2: Capital Return with Multiple Lots**

Shares acquired in two separate purchases, then capital return received:

- Lot 1: 10 shares, cost £1,000
- Lot 2: 10 shares, cost £900
- Total: 20 shares, cost £1,900

Capital return of £100 on 10 shares:

- The £100 reduction applies to the pool proportionally
- Lot 1 adjustment: £100 × (10/20) = £50 reduction
- Lot 2 adjustment: £100 × (10/20) = £50 reduction
- New pool cost: £1,900 - £100 = £1,800
- Average cost: £1,800 / 20 = £90 per share

Note: For Section 104 pooling, all shares are fungible. The capital return reduces the total pool cost regardless of which specific shares the return nominally applies to.

### Accumulation Fund Dividends (ACCUMULATION)

When an accumulation fund reinvests dividends, the `ACCUMULATION` keyword records the basis adjustment:

- The reinvested dividend increases the cost base of your holding
- This is because the dividend is "notionally" reinvested, acquiring additional value
- The increase is apportioned across lots based on shares held at dividend date

Ordinary cash dividends use the `DIVIDEND` keyword instead. Cash dividends do not affect the CGT cost basis; they are reported as income.

**Example:**

- 100 shares in accumulation fund, cost £5,000
- Accumulation dividend of £50
- New cost base: £5,000 + £50 = £5,050
- Average cost per share: £50.50

### Carried Losses

Losses can be carried forward indefinitely to offset against future gains. Important points:

- Losses must be reported within 4 years of the tax year they occurred
- Use losses efficiently to preserve the Annual Exempt Amount
- Losses cannot create or increase a refund

---

## SA108 Tax Return Reporting

**HMRC Reference:** SA108 (Capital Gains Tax Summary)

When completing your Self Assessment tax return, you need to report capital gains using specific values in the SA108 supplementary pages.

### Key Boxes

- **Box 21 (Disposal proceeds)**: The **gross** amount received from selling the asset, before deducting any sale expenses. This is quantity × sale price.
- **Box 22 (Allowable costs)**: All allowable costs including:
  - Original purchase price
  - Purchase fees (broker commission, stamp duty)
  - Sale fees (broker commission)
  - Any enhancement expenditure

### Gross vs Net Proceeds

It's important to understand the difference:

- **Gross Proceeds** = Quantity × Sale Price (goes in SA108 Box 21)
- **Net Proceeds** = Gross Proceeds - Sale Fees (used internally for gain calculation)
- **Gain/Loss** = Net Proceeds - Allowable Cost

**Example:**

- Sell 100 shares at £10.00 each with £12.50 broker fee
- Gross Proceeds: 100 × £10.00 = **£1,000.00** (Box 21)
- Net Proceeds: £1,000.00 - £12.50 = £987.50
- If cost basis was £800.00 with £10.00 purchase fees:
  - Allowable Costs: £800.00 + £10.00 + £12.50 = **£822.50** (Box 22)
  - Gain: £987.50 - £810.00 = £177.50 (or equivalently: £1,000.00 - £822.50)

Note: The sale fee (£12.50) is included in Box 22 (Allowable costs), NOT deducted from Box 21 (Disposal proceeds).

### This Tool's Output

The cgt-tool reports show:

- **Summary table**: Gross proceeds per tax year (for SA108 Box 21)
- **Disposal details**: Both gross and net proceeds for transparency
- **Cost**: The allowable cost used in gain calculation
- **Gains/Losses**: Totals are net per disposal after applying matching rules (CG51560) and allowable costs per TCGA92/S38 (CG15150, CG15250). A disposal can include multiple match legs, but it contributes a single net result to total gain or total loss.

---

## Tax Year Boundaries

The UK tax year runs from **6 April to 5 April**.

- 2023/24 tax year: 6 April 2023 to 5 April 2024
- 2024/25 tax year: 6 April 2024 to 5 April 2025

Gains and losses are calculated and reported per tax year. The Section 104 pool carries forward across years without triggering any tax event.

**Example:** Tax year allocation

- Transaction on 5 April 2024: Falls in 2023/24 tax year
- Transaction on 6 April 2024: Falls in 2024/25 tax year

---

## References

### HMRC Capital Gains Manual

- [CG14250 - Date of Disposal: Conditional Contracts](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg14250)
- [CG51500 - Share Identification: Introduction](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51500)
- [CG51560 - Same Day and Bed & Breakfast Rules](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560)
- [CG51575 - Section 104 Holdings](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51575)
- [CG51590 - Share Identification: Examples](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51590)
- [CG51746 - Bonus and Rights Issues](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51746)
- [CG58620 - Capital Returns](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg58620)
- [CG57835 - Small Capital Distributions: Introduction](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57835)
- [CG57844 - Small Capital Distributions: Section 104 Computation](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57844)
- [CG57847 - Small Capital Distributions: Proceeds More Than Allowable Expenditure](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57847)

### HMRC Employment-Related Securities Manual

- [ERSM20192 - Restricted Stock Units: Vesting](https://www.gov.uk/hmrc-internal-manuals/employment-related-securities/ersm20192)

### HMRC Helpsheets

- [HS284 - Shares and Capital Gains Tax](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet)
- [HS285 - Share Reorganisations and CGT](https://www.gov.uk/government/publications/share-reorganisations-company-takeovers-and-capital-gains-tax-hs285-self-assessment-helpsheet)

### Annual Exempt Amount

- [Capital Gains Tax allowances](https://www.gov.uk/capital-gains-tax/allowances)

### Allowable Expenses

- [CG15250 - Allowable Incidental Costs](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg15250)

---

**Document Version:** 1.4
**Last Updated:** 2026-02-23
**Source:** HMRC Capital Gains Manual (CG51500-CG51600), HMRC Employment-Related Securities Manual (ERSM20192)
