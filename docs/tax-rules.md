# UK Capital Gains Tax: share matching rules

This document explains the share matching rules and tax assumptions used by cgt-tool.
A disposal is a sale or other transfer of an asset.
An acquisition is a purchase or other receipt of an asset.
Matching identifies the acquisition cost to deduct when calculating a disposal gain or loss.

Related documents: [behavioral specification](spec.md) and [architecture](architecture.md).
For commands and transaction syntax, see the [README](../README.md).

## Scope and assumptions

The input represents one owner in one ownership capacity.
Shares of the same class in the same company belong together, including shares held through different brokers.
The tool groups assets by ticker, so consistent identifiers matter.
It assumes that the UK residence conditions for the 30-day rule apply.
HMRC assesses residence at the later acquisition date, as explained in [CG51560](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560).

Include all earlier acquisitions and transactions through 30 days after each reported sale.
The tool cannot identify a missing acquisition from an incomplete history.
It rejects short sales and consolidations that leave fractional actual shares.
The [specification](spec.md#validation-and-errors) describes these product restrictions.

The report applies same-year losses and the annual exemption.
It excludes losses brought forward and other reliefs and does not calculate the final tax bill.
`CAPRETURN` assumes that a distribution qualifies for [cost reduction](#capital-returns), without assessing that qualification.

## Matching order

For disposals from 6 April 2008 onward, the supported matching order is:

1. Same Day: Shares acquired on the disposal date.
2. Bed & Breakfast: Shares acquired within the following 30 days.
3. Section 104 Pool: The remaining pooled holding at average cost.

The Same Day rule takes priority even when an earlier disposal competes for a later acquisition.
Shares matched under Same Day or Bed & Breakfast do not enter the Section 104 pool.
The examples below assume no other transactions that change the stated matches.
See [CG51560](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560) and [CG51575](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51575).

## Same Day matching

HMRC combines acquisitions and disposals by date, share class, owner, and ownership capacity under TCGA92/S105(1)(a).
It matches the disposal against that day's acquisition first under S105(1)(b).
Transaction order within the day does not change this priority.
Purchase costs include allowable expenses, and partial matches receive a proportional cost.

### Full and partial matches

On 15 August 2023, buy 100 shares at £10 each with £10 expenses.
Sell those 100 shares at £12 each with £10 sale expenses.
The calculation is:

| Amount           | Calculation              |
| ---------------- | ------------------------ |
| Acquisition cost | 100 × £10 + £10 = £1,010 |
| Net proceeds     | 100 × £12 − £10 = £1,190 |
| Gain             | £1,190 − £1,010 = £180   |

No shares enter the Section 104 pool.
For a partial match, consider these transactions on 20 September 2023:

| Transaction                             | Amount                       |
| --------------------------------------- | ---------------------------- |
| Buy 200 shares at £5 with £20 expenses  | £1,020 acquisition cost      |
| Sell 150 shares at £6 with £15 expenses | £885 net proceeds            |
| Match 150 of the 200 shares             | £1,020 × 150/200 = £765 cost |
| Gain                                    | £885 − £765 = £120           |
| Remaining pool                          | 50 shares with £255 cost     |

### Same Day and pooled shares together

Start with a pool of 40 shares costing £7,338.70.
A capital return of £149.75 on 31 May 2023 reduces that cost.
On 5 November 2023, buy 20 shares and sell 40 shares, both at £194.22 per share with no fees.

| Match                 | Cost                                       | Gain                              |
| --------------------- | ------------------------------------------ | --------------------------------- |
| 20 Same Day shares    | 20 × £194.22 = £3,884.40                   | £0                                |
| 20 Section 104 shares | (£7,338.70 − £149.75) × 20/40 = £3,594.475 | £3,884.40 − £3,594.475 = £289.925 |

The complete disposal gain rounds to £289.93.
The remaining pool contains 20 shares with exact cost £3,594.475.
The same-day acquisition supplies only half of the disposal quantity.

## Bed & Breakfast matching

The 30-day rule matches later acquisitions before the Section 104 pool.
The window starts the day after the sale and includes day 30.
Multiple eligible purchases match chronologically, with the earliest purchase first.
For acquisitions from 22 March 2006, the rule does not apply when the disposer is not UK tax resident at acquisition.
See [CG51560](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51560).

The tax pool and actual share ownership can differ during this window.
A later purchase supplies the matched tax cost but does not supply shares held on the earlier sale date.
Each example assumes enough actual shares at the sale date.

### Competing claims

Start with at least 100 shares.
On 1 February, sell 100 shares.
On 2 February, buy 80 shares and sell 50 shares.

The 2 February sale uses 50 shares from that day's purchase.
The 1 February sale can use only the remaining 30 shares under Bed & Breakfast.
Its other 70 shares match against the Section 104 pool.
The priority follows TCGA92/S105(1) and S106A(9).

### Full and partial repurchases

On 10 March 2023, sell 100 shares at £8 with £12 expenses.
On 25 March 2023, buy 100 shares at £7.50 with £12 expenses.
The later purchase supplies the cost for all 100 sold shares:

| Amount           | Calculation              |
| ---------------- | ------------------------ |
| Net proceeds     | 100 × £8 − £12 = £788    |
| Acquisition cost | 100 × £7.50 + £12 = £762 |
| Gain             | £788 − £762 = £26        |

A purchase on 11 April is 32 days after 10 March and falls outside the window.
Without another eligible purchase, the sale uses the Section 104 pool.

For a partial repurchase, start with 500 pooled shares costing £2,000, or £4 each.
On 1 June 2023, sell 200 shares at £6 with £20 expenses, for £1,180 net proceeds.
On 15 June, buy 100 shares at £5.50 with £10 expenses, for £560 cost.

| Match                      | Net proceeds            | Cost            | Gain |
| -------------------------- | ----------------------- | --------------- | ---- |
| 100 Bed & Breakfast shares | £1,180 × 100/200 = £590 | £560            | £30  |
| 100 Section 104 shares     | £590                    | 100 × £4 = £400 | £190 |
| Total                      | £1,180                  | £960            | £220 |

After the repurchase, the pool contains 400 shares costing £1,600, still £4 each.
Only the 100 Section 104 shares reduce the pool.
The 100 repurchased shares already match the earlier disposal.

## Section 104 pooling

The pool records a share quantity and total allowable acquisition cost.
It combines shares of the same class in the same company held by one owner in one capacity.
Unmatched purchases increase both totals, and Section 104 disposals remove a proportional cost.
The pool continues across tax years.
See [CG51575](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51575).

```text
Average cost per share = Total pool cost / Total pool shares
Cost allocated to disposal = Total pool cost × Shares matched / Total pool shares
```

Use the exact ratio for the calculation.
Round the final disposal amount, not the intermediate unit cost.
The [specification](spec.md#foreign-exchange-and-precision) defines the tool's rounding behavior.

### Partial disposal

These purchases create a pool of 150 shares costing £1,625:

| Date            | Transaction                      | Amount              |
| --------------- | -------------------------------- | ------------------- |
| 1 January 2022  | Buy 100 at £10 with £15 expenses | £1,015 cost         |
| 1 June 2022     | Buy 50 at £12 with £10 expenses  | £610 cost           |
| 1 December 2022 | Sell 80 at £15 with £12 expenses | £1,188 net proceeds |

The unit cost is £1,625 / 150, or approximately £10.833333 per share.
The disposal cost is £1,625 × 80/150, which rounds to £866.67.
The gain is £1,188 − (£1,625 × 80/150), which rounds to £321.33.

The remaining pool contains 70 shares with cost £1,625 × 70/150, displayed as £758.33.
The pool retains the unrounded remainder.
Its average cost per share remains £1,625 / 150.

### Pool across tax years

In 2021/22, buy 200 shares on 1 May 2021 at £5, with total cost £1,020 including expenses.
The pool contains 200 shares at £5.10 each.
In 2022/23, the following transactions change it:

| Date            | Transaction                                           | Pool or result                        |
| --------------- | ----------------------------------------------------- | ------------------------------------- |
| 1 August 2022   | Buy 100 shares at £6, costing £615 including expenses | 300 shares, £1,635 cost, £5.45 each   |
| 1 February 2023 | Sell 150 shares at £7, with £1,035 net proceeds       | £817.50 matched cost and £217.50 gain |
| After sale      | Retain 150 shares                                     | £817.50 pool cost, £5.45 each         |

The same 150 shares and £817.50 cost carry into 2023/24.
The tax-year boundary does not trigger a disposal.
Later purchases or corporate actions can change the pool.

## Corporate actions and employee shares

### RSU acquisitions

RSU means Restricted Stock Unit.
An award can deliver shares or a cash equivalent.
Its terms determine the tax treatment, as explained in [ERSM20192](https://www.gov.uk/hmrc-internal-manuals/employment-related-securities/ersm20192).

The Schwab converter assumes supported awards deliver shares on vesting.
It uses the vest date as the acquisition date and the vest-date fair market value as acquisition cost.
A later broker deposit date does not replace that date.
This assumption does not cover every award arrangement.
[CG14250](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg14250) explains how contractual conditions affect acquisition and disposal dates.

For example, 30 shares vest on 15 January 2024 and reach the broker account on 17 January.
The employee sells 30 shares on 15 January.
Using the vest date produces a Same Day match.

Using 17 January instead introduces an acquisition within the 30-day window.
With sufficient earlier holdings, the tool then uses Bed & Breakfast.
Without those holdings, the tool rejects the sale.
The wrong date therefore changes matching or causes an error, rather than automatically selecting Section 104.
See [converter behavior](spec.md#awards-and-acquisition-dates) for award lookup and missing-data rules.

### Splits, consolidations, and bonus shares

A qualifying reorganization changes share quantities without creating a matching acquisition or disposal under TCGA92/S127.
For a split or same-class bonus issue, total pool cost stays the same while quantity increases.
For a consolidation, quantity decreases and average cost increases.
See [CG51746](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51746) and [HS285](https://www.gov.uk/government/publications/share-reorganisations-company-takeovers-and-capital-gains-tax-hs285-self-assessment-helpsheet).

A 2-for-1 split changes 100 shares costing £1,000 into 200 shares costing £1,000.
Average cost falls from £10 to £5.
The tool supports quantity changes through `SPLIT` and `UNSPLIT`, subject to its [input restrictions](spec.md#validation-and-errors).

### Capital returns

A capital distribution normally requires consideration of a disposal under TCGA92/S122.
A small distribution can instead reduce allowable share cost under S122(2).
See [CG58620](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg58620) and [CG57835](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57835).

HMRC normally accepts a distribution at or below 5% of the holding value immediately before distribution, or £3,000.
Other cases depend on their facts.
`CAPRETURN` does not assess these conditions and assumes that cost reduction is the correct treatment.
The tool reduces pool cost by the supplied distribution less its fees.

For 100 shares costing £800, a qualifying return of £2 per share with no fees totals £200.
The new pool cost is £800 − £200 = £600.
Share quantity stays at 100.

For a second example, two purchases of 10 shares cost £1,000 and £900.
The pool contains 20 shares costing £1,900.
A qualifying £100 capital return recorded against 10 shares reduces the combined pool cost:

| Amount                                      | Calculation            |
| ------------------------------------------- | ---------------------- |
| Proportional illustration for each purchase | £100 × 10/20 = £50     |
| Remaining pool cost                         | £1,900 − £100 = £1,800 |
| Average cost                                | £1,800 / 20 = £90      |

The pool treats its shares as interchangeable and does not retain separate purchase lots for this adjustment.
The distribution reduces the total pool cost.
[CG57844](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57844) explains the Section 104 treatment.

If a small distribution exceeds allowable cost, S122(2) does not apply.
S122(1) provides part-disposal treatment, and S122(4) permits an election for a small distribution to use the remaining allowable cost.
The tool supports neither treatment and rejects a `CAPRETURN` that reduces pool cost below zero.
See [CG57847](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg57847).

### Accumulation and cash dividends

Accumulation units retain income in the fund instead of paying it as cash.
Taxable notional distributions count as income and additional allowable expenditure.
See [HS284, accumulation units](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet/hs284-shares-and-capital-gains-tax-2025#accumulation-units).

`ACCUMULATION` increases pool cost without changing share quantity.
For 100 shares costing £5,000, a £50 accumulation distribution increases cost to £5,050, or £50.50 per share.
A positive-quantity record also contributes its distribution and tax paid to annual income totals.
A zero-quantity record has no effect.
Cash dividends use `DIVIDEND` and contribute income without changing pool cost.

## Losses and tax years

The UK tax year runs from 6 April through 5 April.
The 2023/24 year spans 6 April 2023 through 5 April 2024.
The 2024/25 year starts on 6 April 2024 and ends on 5 April 2025.
A sale on 5 April 2024 belongs to 2023/24, and a sale the next day belongs to 2024/25.

Same-year losses reduce gains before the annual exemption.
Claimed unused losses from earlier years reduce remaining gains only to the exemption, with unused losses carried forward.
A loss claim normally has a four-year deadline after the disposal tax year.
See [HMRC loss guidance](https://www.gov.uk/capital-gains-tax/losses) and [annual allowances](https://www.gov.uk/capital-gains-tax/allowances).

The tool applies same-year losses and the configured exemption.
It does not track claims, losses used elsewhere, or losses brought forward.
Its `taxable_gain` therefore precedes those adjustments and other reliefs.

## SA108 reporting

SA108 is the Capital Gains Tax summary for Self Assessment.
Use the form and notes for the relevant tax year and asset category.
The report does not split totals by SA108 category, so a mixed-asset report requires separate classification.

For ordinary listed shares and securities, these mappings apply to the [2024/25 form](https://assets.publishing.service.gov.uk/media/67e160d5d8e313b503358cc8/sa108-2025.pdf) and [2025/26 form](https://assets.publishing.service.gov.uk/media/69bd8990cfa346b9d47049e4/SA108-2026.pdf):

| Box | Meaning                                                                 |
| --- | ----------------------------------------------------------------------- |
| 23  | Number of disposals                                                     |
| 24  | Gross disposal proceeds, before sale expenses                           |
| 25  | Allowable costs, including acquisition cost and allowable sale expenses |
| 26  | Gains before losses                                                     |
| 27  | Losses                                                                  |

Other categories and relief claims can require different sections.
Box numbers alone are not a universal mapping for every report year.
Report notes explain the amounts without fixed box references.
Use the form's box labels when transferring report amounts.
The [SA108 publication page](https://www.gov.uk/government/publications/self-assessment-capital-gains-summary-sa108) provides forms and notes by year.

The report's `Cost` and JSON `total_allowable_cost` contain matched acquisition costs, including purchase expenses.
They exclude sale expenses because the gain calculation already deducts those from gross proceeds.
SA108 allowable costs also include allowable sale expenses, so the report cost alone is not the complete box 25 amount.
See [CG15250](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg15250) for allowable incidental costs.

```text
Net proceeds = Gross proceeds − Sale expenses
Gain or loss = Net proceeds − Matched acquisition cost
SA108 allowable costs = Matched acquisition cost + Allowable sale expenses
Gain or loss = Gross proceeds − SA108 allowable costs
```

For example, sell 100 shares at £10 each with a £12.50 broker fee.
The matched acquisition cost includes an £800 purchase price and £10 purchase fees.
The two gain formulas give the same result:

| Amount                   | Calculation                   |
| ------------------------ | ----------------------------- |
| Gross proceeds           | 100 × £10 = £1,000.00         |
| Net proceeds             | £1,000.00 − £12.50 = £987.50  |
| Matched acquisition cost | £800 + £10 = £810.00          |
| SA108 allowable costs    | £810.00 + £12.50 = £822.50    |
| Gain                     | £987.50 − £810.00 = £177.50   |
| Equivalent gain          | £1,000.00 − £822.50 = £177.50 |

The report groups same-day disposals and assigns each complete disposal to either gains or losses.
Several match legs within one disposal do not create separate gain and loss entries.
For further worked examples, see [CG51590](https://www.gov.uk/hmrc-internal-manuals/capital-gains-manual/cg51590), [HS284](https://www.gov.uk/government/publications/shares-and-capital-gains-tax-hs284-self-assessment-helpsheet), and the [report specification](spec.md#reports).
