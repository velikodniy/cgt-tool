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

**HMRC Reference:** CG51560

### Definition

If you buy and sell shares of the same class in the same company on the same day, the sale is matched first against shares bought on that day.

### Key Points

- All purchases on the same day are aggregated (treated as one transaction)
- All sales on the same day are aggregated (treated as one disposal)
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

- Reduces the cost base of your holding
- May trigger a deemed disposal if return exceeds cost base

**Example:** £2.00 per share capital return on 100 shares (pool cost £800):

- Return: 100 × £2.00 = £200
- New pool cost: £800 - £200 = £600
- If return exceeded £800, the excess would be a gain

### Carried Losses

Losses can be carried forward indefinitely to offset against future gains. Important points:

- Losses must be reported within 4 years of the tax year they occurred
- Use losses efficiently to preserve the Annual Exempt Amount
- Losses cannot create or increase a refund

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

- [CG51500 - Share Identification: Introduction](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51500)
- [CG51560 - Same Day and Bed & Breakfast Rules](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560)
- [CG51575 - Section 104 Holdings](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51575)
- [CG51590 - Share Identification: Examples](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51590)
- [CG51746 - Bonus and Rights Issues](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51746)
- [CG58620 - Capital Returns](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg58620)

### HMRC Helpsheets

- [HS284 - Shares and Capital Gains Tax](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet)
- [HS285 - Share Reorganisations and CGT](https://www.gov.uk/government/publications/share-reorganisations-company-takeovers-and-capital-gains-tax-hs285-self-assessment-helpsheet)

### Allowable Expenses

- [CG15250 - Allowable Incidental Costs](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg15250)

---

**Document Version:** 1.0
**Last Updated:** 2025-12-08
**Source:** HMRC Capital Gains Manual (CG51500-CG51600)
