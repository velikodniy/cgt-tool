# Behavioral specification

This document defines accepted inputs, calculation behavior, and report fields.
It describes the current implementation, including product restrictions.
The [tax rules](tax-rules.md#scope-and-assumptions) define the supported tax context and required transaction history.

For usage, see the [README](../README.md).
For internal design and testing methods, see [architecture](architecture.md).

## Transaction input

The DSL is a line-based format for transactions.
Each line contains one transaction or a comment.
The CLI combines multiple input files before calculation, so overlapping exports duplicate transactions.

In the formats below, angle brackets mark values and square brackets mark optional clauses:

| Type          | Format                                                                                           |
| ------------- | ------------------------------------------------------------------------------------------------ |
| BUY/SELL      | `<date> BUY\|SELL <ticker> <quantity> @ <price> [<currency>] [FEES <amount> [<currency>]]`       |
| DIVIDEND      | `<date> DIVIDEND <ticker> TOTAL <value> [<currency>] [TAX <amount> [<currency>]]`                |
| ACCUMULATION  | `<date> ACCUMULATION <ticker> <quantity> TOTAL <value> [<currency>] [TAX <amount> [<currency>]]` |
| CAPRETURN     | `<date> CAPRETURN <ticker> <quantity> TOTAL <value> [<currency>] [FEES <amount> [<currency>]]`   |
| SPLIT/UNSPLIT | `<date> SPLIT\|UNSPLIT <ticker> RATIO <ratio>`                                                   |

Dates use `YYYY-MM-DD`.
Keywords accept either case.
An amount without an ISO 4217 currency code uses GBP.
Each price, fee, or tax amount has its own optional currency code.
Omitted `FEES` and `TAX` clauses default to zero GBP.
A `#` starts a comment, including after transaction data.

Tickers start with an ASCII letter or digit.
Later characters can also include `.`, `/`, `-`, and `_`.
The parser converts letters to uppercase and preserves punctuation, so `BRK/B` and `BRK.B` identify separate assets.
The converter uses the same symbol rules.

`cgt-tool parse --schema` returns the JSON schema for input transactions.
This schema describes transactions, not the report JSON.
The CLI and WebAssembly functions read DSL text.
Rust callers can supply typed transactions or deserialize transaction JSON before calculation.

## Validation and errors

Parsing errors identify the input line and the syntax or value problem.
Shared validation applies to DSL records, JSON deserialization, and calculation with direct Rust input.
Rules that involve multiple records apply before calculation.

| Field or combination                                    | Accepted behavior                                             |
| ------------------------------------------------------- | ------------------------------------------------------------- |
| BUY/SELL quantity                                       | Must be positive                                              |
| Prices, fees, distribution values, and tax paid         | Must be nonnegative                                           |
| SELL at zero price                                      | Uses normal matching and reports the resulting gain or loss   |
| ACCUMULATION quantity                                   | Can be zero, which has no effect on cost, income, or tax paid |
| CAPRETURN quantity                                      | Must be positive                                              |
| SPLIT ratio                                             | Must be positive                                              |
| UNSPLIT ratio                                           | Must be a positive integer and leave whole actual shares      |
| SPLIT/UNSPLIT with BUY/SELL on the same date and ticker | Fails as ambiguous input                                      |
| Tax-year start                                          | Must be within 1900 through 2100                              |
| Annual exemption                                        | Must be nonnegative                                           |

The same-date split restriction reflects the input format, which cannot express an order within a day.
Applying a trade before or after the split can change the holding.
Keep the actual transaction dates and calculate unsupported cases separately.
`CAPRETURN`, `ACCUMULATION`, and `DIVIDEND` do not change share counts, so this restriction does not apply to them.

For parsed transactions and a loaded configuration, calculation follows this error order:

1. Input validation rejects invalid fields or conflicting records.
2. Normalization reports missing exchange rates or unsupported trade arithmetic.
3. Quantity planning reports insufficient holdings or invalid consolidation quantities.
4. Valuation reports invalid cost adjustments or unsupported monetary arithmetic.
5. Report assembly reports invalid tax years, exemption errors, or unsupported report totals.

The first failed stage stops calculation without a partial report.
Arithmetic errors name the calculation and its transaction when applicable.
A nonzero share quantity that becomes zero at the public decimal boundary also returns an error.

## Calculation

### Matching and holdings

The matching order is Same Day, then Bed & Breakfast within 30 days, then the Section 104 Pool.
A match leg is a portion of a disposal assigned to one rule.
Each ticker has a separate pool.

Same Day matching combines all purchases and all sales for each date and ticker.
Unrelated transactions between those rows do not split the groups.
The matched quantity is the smaller group total, and excess sales continue to the next rule.

Bed & Breakfast matches later purchases chronologically.
Day D+30 is inside the window, and D+31 is outside it.
A future purchase first reserves the shares required by sales on its own date.
Earlier sales can use only the remainder, even when the input contains several lots for that date and ticker.

Section 104 matches the remaining disposal quantity against the pool at average cost.
Only purchases not allocated to Same Day or Bed & Breakfast matches enter the pool.
Only Section 104 matches reduce it.
For example, a sale of 100 shares with a later matching purchase of 40 uses 40 Bed & Breakfast shares and 60 pooled shares.

The tool tracks actual shares separately from the tax pool.
Same-day purchases increase actual holdings before sales, and every sale reduces them.
A later purchase restores actual shares only on its acquisition date.
Sales that exceed actual holdings fail, even if a future purchase supplies enough shares for tax matching.
This product restriction excludes short sales.

### Corporate actions and income

`SPLIT` multiplies share quantities by its ratio without changing total cost.
`UNSPLIT` divides quantities by its ratio without changing total cost.
These reorganizations do not count as acquisitions or disposals for matching.
Split ratios also apply across the 30-day window, with exact quantities preserved until conversion to public decimals.
The whole-share restriction for `UNSPLIT` applies to actual holdings, not the tax pool.

`CAPRETURN` reduces the existing pool cost by `TOTAL − FEES`, after currency conversion.
The tool assumes that this distribution qualifies for cost reduction under the [documented tax limits](tax-rules.md#capital-returns).
If the net return exceeds the remaining basis, calculation fails with an error referencing TCGA92/S122 and CG57847.
A positive net return against a fully disposed or never-held pool also fails.
The tool does not implement part-disposal treatment under S122(1) or elections under S122(4).

`ACCUMULATION` adds its `TOTAL` value to the existing pool cost.
Its distribution and tax paid also contribute to annual income totals.
A positive distribution without a pool holding fails.
A zero-quantity record has no effect and requires neither a holding nor an exchange rate.
Input fields must still satisfy the nonnegative-value rules.

`DIVIDEND` adds cash income and tax paid to annual totals without changing pool cost.
For each date, value events apply before trades in this order: `CAPRETURN`, `ACCUMULATION`, then `DIVIDEND`.
Later capital returns and accumulation dividends adjust the carried-forward pool without changing earlier disposal costs.
This also applies after a day containing both purchases and sales.

### Foreign exchange and precision

Foreign amounts use HMRC monthly exchange rates for the transaction month.
The [`hmrc-rates`](https://crates.io/crates/hmrc-rates) dependency bundles the monthly series from February 2014 onward at build time.
GBP amounts need no conversion.
A missing currency/month pair produces `MissingFxRate`, with no fallback to another month.
If the required month is published, install a release that includes its rates.

Trades retain their gross totals and fees at the available decimal precision.
Partial matches receive proportional costs, proceeds, and fees.
The final allocation receives the complete remainder.
Unit prices are derived for output and do not determine allocation precision.

Each disposal rounds its complete cost and gain once, after summing its precise legs.
Annual totals sum the rounded disposal totals.
Money rounds to two decimal places with midpoints away from zero, so £100.995 becomes £101.00.
Holding quantities and costs retain full decimal precision in JSON.

Displayed legs receive differences between consecutive rounded totals, so their sums equal the disposal totals.
Independently rounded proceeds, costs, and gains can differ by one penny.
A nonzero `rounding_adjustment` explains this difference in JSON, plain text, and PDF without changing the calculated gain.

### Tax years and exemptions

Tax years run from 6 April through 5 April.
A disposal on 5 April 2024 belongs to 2023/24, and 6 April 2024 starts 2024/25.
Without a year filter, reports include years with disposals or income in chronological order.
`--year 2024` returns a 2024/25 summary, including when that year has no activity.
All input transactions still participate in matching and final holdings.

Each disposal contributes one net gain or loss, regardless of its match count.
A zero net result contributes to neither gains nor losses.
The calculation uses these relationships:

```text
net_proceeds = gross_proceeds − sale_expenses
gain_or_loss = net_proceeds − matched_acquisition_cost
total_gain = sum of positive disposal gains
total_loss = sum of absolute disposal losses
net_gain = total_gain − total_loss
taxable_gain = max(net_gain − exempt_amount, 0)
```

The report excludes losses brought forward and other reliefs.
It does not calculate tax payable.
See [tax rules](tax-rules.md#losses-and-tax-years) for the distinction.

The CLI loads exemptions in this order, with later entries overriding earlier entries:

1. [Embedded defaults](../crates/cgt/data/config.toml).
2. `./config.toml` in the working directory.
3. `~/.config/cgt-tool/config.toml`, when `HOME` is available.

Each file uses an `[exemptions]` table with quoted tax-year start years and GBP amounts.
For example, this entry applies to 2024/25:

```toml
[exemptions]
"2024" = 3000
```

The CLI skips an unreadable or invalid override file with a warning.
The library rejects invalid overrides without changing the existing configuration.
A missing exemption stops report generation unless `--allow-missing-exemption` requests zero allowance with a warning.
This flag does not permit an invalid exemption value.

## Reports

### JSON fields

Report calculations use GBP.
Money and quantities are decimal strings, and `disposal_count` is an integer.
Money rounds to two decimal places but does not require trailing zeros.
Dates use `YYYY-MM-DD`, and tax periods use `YYYY/YY`.
The [report model](../crates/cgt/src/report.rs) defines serialization.

| Object           | Fields and meaning                                                                                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Report           | `tax_years` contains annual summaries. `holdings` contains final pools. Optional `transactions` retains the input records. Nonempty `warnings` contains report warnings. |
| Annual summary   | `period`, `disposals`, and `disposal_count` identify the tax year and grouped disposals.                                                                                 |
| Annual gains     | `total_gain`, `total_loss`, and `net_gain` follow the formulas above.                                                                                                    |
| Annual amounts   | `gross_proceeds` precedes sale fees. `total_allowable_cost` sums matched acquisition costs, including purchase fees but excluding sale fees.                             |
| Annual exemption | `exempt_amount` is the configured allowance. `taxable_gain` precedes losses brought forward and other reliefs.                                                           |
| Annual income    | `dividend_income` and `dividend_tax_paid` include cash dividends and nonzero-quantity accumulation records.                                                              |
| Disposal         | `date`, `ticker`, `quantity`, `gross_proceeds`, `proceeds`, and `matches` describe one grouped sale. `proceeds` is net of sale fees.                                     |
| Match            | `rule`, `quantity`, `allowable_cost`, and `gain_or_loss` describe one displayed leg. `rule` is `SameDay`, `BedAndBreakfast`, or `Section104`.                            |
| Match date       | `acquisition_date` appears for Same Day and Bed & Breakfast matches and is absent for Section 104.                                                                       |
| Rounding         | A disposal includes `rounding_adjustment` only when the display difference is nonzero.                                                                                   |
| Holding          | `ticker`, `quantity`, and `total_cost` describe a final Section 104 pool. Quantities and costs retain full precision.                                                    |

`holdings` reflects all supplied transactions, including those outside `--year`.
It is not a snapshot at the selected tax-year end.
JSON can include fully depleted pools with zero quantity.
It does not store a separate average cost.

The transaction records retain original currency amounts and input order.
GBP calculations appear in the disposal and annual fields.
The tool does not classify assets into SA108 categories or produce a complete tax return.
In particular, `total_allowable_cost` excludes sale expenses required in the [SA108 cost total](tax-rules.md#sa108-reporting).

### Prepared display document

The [`report_display::prepare`](../crates/cgt/src/report_display.rs) function converts a calculation report into an immutable `ReportDisplay`.
Plain text and PDF renderers accept this prepared document.
The WASM `calculate_display` function returns its serialized form for the browser.
The existing `calculate_tax` function continues to return calculation JSON.

The display document contains formatted annual summaries, disposals, positive holdings, sorted trade echoes, asset events, selected notes, and warnings.
Money fields contain GBP text with grouping and two decimal places.
Quantities remain exact decimal text without trailing zeros.
Dates contain `iso` and `uk` text.
Unit costs contain `money` text and ungrouped `compact` text for plain output.
Disposal results contain `kind`, `signed`, and `magnitude` fields.
Optional fees, rounding adjustments, dividend sections, and tax values resolve their relevance during preparation.
Note identifiers remain stable while each output chooses its layout and numbering.

Preparation leaves the calculation report unchanged and reuses its cumulative match rounding.
It derives sale unit prices from precise gross proceeds, before calculation JSON rounds those proceeds.
It derives holding averages from exact costs and quantities.
Zero disposal results use the gain classification.
Preparation returns an arithmetic error instead of a partial document when a directly constructed report exceeds supported arithmetic.
This includes sums and differences that decimal arithmetic accepts after silently discarding fractional digits.
A zero disposal quantity retains the existing zero unit-price convention.

### Plain text and PDF

Both formats contain a summary, disposal details, holdings, and transactions.
The disposal section is titled Tax Year Details in plain text and Disposal Details in PDF.
Asset Events appears when the input contains dividends or corporate actions.
Transactions and asset events preserve original currencies and sort by date, then ticker.
Disposal calculations use GBP.

Monetary totals use the £ symbol, comma thousands separators, and two decimal places, such as £1,234.00 or -£100.00.
Dates use `DD/MM/YYYY`.
Trade unit prices retain decimal precision without trailing zeros.
Holding unit costs derive from exact pool totals and round to two decimal places for display.
Only positive holdings appear.
An empty holding list shows `NONE` in plain text and `No remaining holdings.` in PDF.

Annual summaries show grouped disposal counts, gains before and after losses, losses, gross proceeds, exemptions, and taxable gains.
They also show dividend income and tax paid when applicable.
Report notes explain grouped disposals, net gains and losses, gross proceeds, and the limits of taxable gain.
The same relevant notes appear in plain text, PDF, and the browser, including single-year browser reports.
Disposal notes appear only when the selected report contains disposals.
The taxable gain note appears when the report contains annual summaries.
Notes explain report amounts without fixed SA108 box references.
Use the form for the relevant asset category and tax year.

Disposal details show `quantity × unit_price = gross_proceeds`.
When fees exist, a second line subtracts fees from gross proceeds to show net proceeds.
Displayed fees equal rounded gross proceeds minus rounded net proceeds.
A positive precise fee retains its line even when its displayed amount equals £0.00.
Same-day merged disposals derive their unit price from total gross proceeds divided by total quantity.

Plain text and JSON default to standard output, with `--output` selecting a file.
PDF uses embedded Typst and bundled fonts, without external rendering tools.
Its summary headers use superscript footnote markers.
For one input file, the default PDF path replaces its extension with `.pdf`.
For multiple input files, the default is `report.pdf` in the working directory.
The CLI refuses to overwrite an existing default PDF path, while explicit `--output` paths permit replacement.

## Broker conversion

### Schwab records

The converter reads the `BrokerageTransactions` array from Schwab JSON exports.
It produces chronological DSL records and a header with the skipped-record count.
Every physical comment line starts with `#`, and output contains no conversion timestamp.
See the [README](../README.md#charles-schwab) for export and conversion commands.

| Record                                                                     | Treatment                                                  |
| -------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Buy and Sell                                                               | Become BUY and SELL records                                |
| Stock Plan Activity                                                        | Uses the awards export for acquisition dates and values    |
| Cash Dividend, Qualified Dividend, Short Term Cap Gain, Long Term Cap Gain | Become DIVIDEND records after correction reconciliation    |
| NRA Withholding and NRA Tax Adj                                            | Adjust dividend tax paid after reconciliation              |
| Cancel Sell                                                                | Removes one matching sale                                  |
| Stock Split                                                                | Produces a comment for manual ratio entry                  |
| Recognized non-CGT cash movements, including Credit Interest and Wire Sent | Are skipped                                                |
| Unknown actions                                                            | Produce warnings and comments instead of conversion errors |

Schwab dates use `MM/DD/YYYY` or `as of` notation.
For `02/25/2021 as of 02/21/2021`, conversion uses 21 February 2021.
Successful conversion produces parseable DSL, but missing history or unresolved manual actions can still prevent a correct report.

### Awards and acquisition dates

Fair market value (FMV) is the share value at acquisition.
For supported RSU awards, the converter uses the vest date and FMV from the awards file.
This assumes that the award delivers shares on vesting, as described in the [tax rules](tax-rules.md#rsu-acquisitions).

The awards parser reads `VestFairMarketValue` and `VestDate`, with `FairMarketValuePrice` and parent `Date` as fallbacks.
Symbol lookup ignores letter case.
Wire Transfer, Tax Withholding, Tax Reversal, Forced Disbursement, and unknown non-vesting actions can have empty details.
Vesting actions require valid award details.

Conversion first matches the parent deposit date, symbol, and available quantity.
Without a linked deposit, it searches vest dates within the preceding seven days.
Conflicting matching dates or values return an ambiguity error, including conflicting values on one date.
The resulting acquisition date controls Same Day matching and eligibility within 30 days after a sale.

### Corrections and cancellations

Dividend and withholding records require valid string amounts.
Reconciliation preserves signs and groups records by effective date and symbol.
Negative dividend corrections reduce positive credits, and unresolved corrections return an error.
Positive withholding adjustments reduce withheld tax.
Unmatched or symbol-less withholding produces a warning.

Sale cancellations match date, symbol, quantity, price, and available fees.
If missing fees leave different candidate sales, conversion returns an ambiguity error.
A cancellation removes only one matching sale.

## Test references

These inputs cover representative behavior.
Each `.cgt` file has a same-named expected JSON file in [`tests/json/`](../tests/json/) and plain-text file in [`tests/plain/`](../tests/plain/).
The [testing policy](architecture.md#testing-and-cross-validation) covers independent numerical evidence and external comparisons.

| Behavior                                | Representative inputs or tests                                                                                                                                                                           |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Same-day grouping and reservations      | [SameDayReservation](../tests/inputs/SameDayReservation.cgt), [SameDayMerge](../tests/inputs/SameDayMerge.cgt), [SameDayMergeInterleaved](../tests/inputs/SameDayMergeInterleaved.cgt)                   |
| Thirty-day boundary and partial matches | [BnBBoundary30Days](../tests/inputs/BnBBoundary30Days.cgt), [PartialBnBWithS104](../tests/inputs/PartialBnBWithS104.cgt)                                                                                 |
| Split ratios across future matches      | [single_pass_bnb_split](../tests/inputs/single_pass_bnb_split.cgt), [single_pass_future_consumption](../tests/inputs/single_pass_future_consumption.cgt)                                                 |
| Actual holdings and chronological order | [physical_holdings.rs](../crates/cgt/tests/physical_holdings.rs), [single_pass_ordering](../tests/inputs/single_pass_ordering.cgt)                                                                       |
| Split precision                         | [SplitThirds](../tests/inputs/SplitThirds.cgt) retains 26 shares. [UnsplitThirds](../tests/inputs/UnsplitThirds.cgt) retains two.                                                                        |
| Cost precision                          | [ExactCapitalReturn](../tests/inputs/ExactCapitalReturn.cgt) reduces £30.01 to zero. [ExactMergedCost](../tests/inputs/ExactMergedCost.cgt) retains £7.005, displayed as £7.01.                          |
| Later corporate actions                 | [single_pass_corp_action](../tests/inputs/single_pass_corp_action.cgt), [WithAssetEventsSameDay](../tests/inputs/WithAssetEventsSameDay.cgt), [WithAssetEventsBB](../tests/inputs/WithAssetEventsBB.cgt) |
| Foreign exchange and mixed currencies   | [single_pass_fx](../tests/inputs/single_pass_fx.cgt), [MultiCurrencySameDay](../tests/inputs/MultiCurrencySameDay.cgt)                                                                                   |
| Gain aggregation and display rounding   | [NetDisposalTotalsMixed](../tests/inputs/NetDisposalTotalsMixed.cgt), [DisposalRounding](../tests/inputs/DisposalRounding.cgt)                                                                           |
| Multiple years and corporate actions    | [SyntheticComplex](../tests/inputs/SyntheticComplex.cgt), [RealisticMultiYear](../tests/inputs/RealisticMultiYear.cgt)                                                                                   |
| Input errors and exemption behavior     | [validation.rs](../crates/cgt/tests/validation.rs), [semantic_validation.rs](../crates/cgt/tests/semantic_validation.rs), [exemptions.rs](../crates/cgt/tests/exemptions.rs)                             |
| Multiple awards and sale prices         | [Synthetic awards](../tests/schwab/synthetic-awards.json), [synthetic transactions](../tests/schwab/synthetic-transactions.json)                                                                         |
