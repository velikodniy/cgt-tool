# CGT Tool — Behavioral Specification

Consolidated reference for cgt-tool behavioral rules and acceptance criteria.
Covers UK Capital Gains Tax calculation per HMRC guidance CG51500–CG51600.

---

## Matching Rules

The system calculates UK CGT using three matching rules applied in strict order:
**Same Day → Bed & Breakfast (30 days) → Section 104 Pool**.

### Same Day Rule

Disposals are matched with same-day acquisitions first (TCGA92/S105(1)).

- Multiple same-day purchases are aggregated if needed.
- If same-day disposals exceed acquisition quantity, the reservation is capped at the acquisition quantity; excess shares proceed to B&B or S104.
- When multiple buys for the same ticker on the same date are split by unrelated transactions (other tickers), Same Day reservation applies across the aggregate date+ticker quantity, not per-lot.

**Golden file:** `tests/inputs/SameDayReservation.cgt` → `tests/json/SameDayReservation.json`

### Bed & Breakfast Rule

Disposals are matched with acquisitions within 30 calendar days after the sale date (TCGA92/S106A), subject to Same Day reservation priority.

- Match chronologically (earliest purchase first).
- Day D+30 is within the window; D+31 is not.
- Before allowing earlier disposals to B&B-consume a future acquisition, the system reserves shares needed for Same Day matching on that acquisition date.
- Reservation is tracked at the date+ticker level across all same-day lots.
- B&B determines cost basis for a valid disposal — it does not enable disposing of shares the taxpayer does not hold (CG51590 Example 1).
- When a B&B lookahead spans an intervening SPLIT or UNSPLIT, the matcher applies the split ratio when calculating matched quantities.

**Scenarios:**

- Given disposal D1 on Day 1, disposal D2 on Day 2, and acquisition A2 on Day 2 where A2 < D1+D2: D2's Same Day claim is satisfied first; D1's B&B uses only remaining shares.
- Given a disposal with 0 shares held and an acquisition within 30 days: processing fails with an error (B&B does not rescue zero-holding disposals).
- Given a disposal with sufficient S104 holding and a B&B-eligible acquisition: the disposal matches via B&B using the later acquisition's cost, and the S104 pool is not reduced.

**Golden files:**

- `tests/inputs/single_pass_bnb_split.cgt` → `tests/json/single_pass_bnb_split.json`
- `tests/inputs/single_pass_future_consumption.cgt` → `tests/json/single_pass_future_consumption.json`

### Section 104 Pool

Shares not matched by Same Day or B&B are matched against a pooled holding at average cost.

- Separate pools per ticker.
- Purchases add to the pool; sales reduce proportionally.
- If a SELL cannot be fully matched after all three rules, calculation fails with an error indicating the disposal exceeds holdings.

### Holding Verification

Before the matching cascade, the system verifies the seller holds enough shares to cover the disposal. The holding is the sum of same-day acquisitions not yet matched plus the S104 pool quantity for that ticker. If the sell quantity exceeds the holding, the system returns an error.

**Golden files:**

- `tests/inputs/single_pass_ordering.cgt` → `tests/json/single_pass_ordering.json`
- `tests/inputs/single_pass_fx.cgt` → `tests/json/single_pass_fx.json`

### Zero Sell Amount

When a disposal has zero total sell amount, no match result is returned and the system proceeds to the next matching rule without division errors.

---

## Corporate Actions

### Stock Split / Unsplit

- **SPLIT**: Multiply share quantity by the ratio; total cost basis unchanged.
- **UNSPLIT**: Reverse of SPLIT.

### Capital Return (CAPRETURN)

Reduces pool cost basis by the return amount.

- Cost reduction is apportioned across all holding lots proportionally: each lot receives `adjustment × (lot_shares / total_holdings)`.
- Lots acquired after the event date are unaffected.
- If the return exceeds remaining allowable basis, calculation fails with an error referencing TCGA92/S122(2) and CG57847 (part-disposal under S122(1) or election under S122(4) is not currently supported).

### Accumulation Dividends (ACCUMULATION)

Increases pool cost basis by the dividend amount, apportioned across lots in the same proportional manner as CAPRETURN.

### Cash Dividends (DIVIDEND)

Records ordinary cash dividend income. Does not affect Section 104 cost basis. Dividend income and tax paid are aggregated per tax year and reported in the summary.

### Asset Events After Same-Day Buy/Sell

When a CAPRETURN or ACCUMULATION occurs after a date with both BUY and SELL for the same ticker, remaining shares are calculated using CGT matching rules per CG51560: same-day first (TCGA92/S105(1)), then B&B (TCGA92/S106A), then S104.

**Golden file:** `tests/inputs/single_pass_corp_action.cgt` → `tests/json/single_pass_corp_action.json`

---

## Tax Year & Aggregation

### Year Boundaries

UK tax year runs 6 April to 5 April. A disposal on 5 April 2024 belongs to 2023/24; 6 April 2024 starts 2024/25. Valid year range: 1900–2100.

- Calculate for a specific year: return only that year's disposals.
- Calculate for all years: return all years with disposals, sorted chronologically.

### Disposal Count

Derived from the number of grouped disposals after same-day aggregation (CG51560), not stored separately. JSON output includes `disposal_count` with the same value.

**Golden files:**

- `tests/inputs/RealisticMultiYear.cgt` → `tests/json/RealisticMultiYear.json`
- `tests/inputs/MultiCurrencySameDay.cgt` → `tests/json/MultiCurrencySameDay.json`
- `tests/inputs/SameDayMerge.cgt` → `tests/json/SameDayMerge.json`

### Gain/Loss Aggregation

Per TCGA92/S38 (CG15150, CG15250):

- Each disposal's net result (proceeds − allowable cost) contributes to either `total_gain` (net ≥ 0) or `total_loss` (net < 0).
- `net_gain = total_gain − total_loss`.
- A disposal with match legs summing to zero contributes to neither total.
- Multi-currency disposals aggregate from GBP-converted amounts.

**Golden file:** `tests/inputs/NetDisposalTotalsMixed.cgt` → `tests/json/NetDisposalTotalsMixed.json`

### Gain/Loss Calculation

`gain_or_loss = net_proceeds − allowable_cost`, where `net_proceeds = gross_proceeds − sale_expenses`.

---

## FX Conversion

### HMRC Monthly Average Rates

Foreign currency amounts are converted to GBP using HMRC monthly average exchange rates for the transaction month.

### Bundled Rates

Rates from January 2015 to August 2025 are embedded at compile time. No configuration required.

### Custom Rate Folder

`--fx-folder` loads rates from XML files (`YYYY-MM.xml` or `monthly_xml_YYYY-MM.xml`). Custom rates take precedence over bundled when both exist.

### Precision

6 decimal places internally for FX calculations. Display rounds to 2 decimal places using midpoint-away-from-zero rounding.

### Dual Display

Foreign currency transactions display as `£118.42 (150 USD)` in text output. JSON includes both GBP and original currency.

### GBP Default

Amounts without a currency code are treated as GBP. No conversion is performed.

### Missing Rate Errors

When a currency/month rate is unavailable, the system fails with a clear error identifying the missing currency and month, with guidance on resolution. The `has_currency` check queries actual cache contents (not a hardcoded range).

---

## DSL Syntax

### Transaction Types

| Type          | Format                                                                                   |
| ------------- | ---------------------------------------------------------------------------------------- |
| BUY/SELL      | `YYYY-MM-DD BUY\|SELL TICKER QUANTITY @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]`       |
| DIVIDEND      | `YYYY-MM-DD DIVIDEND TICKER TOTAL VALUE [CURRENCY] [TAX AMOUNT [CURRENCY]]`              |
| ACCUMULATION  | `YYYY-MM-DD ACCUMULATION TICKER QUANTITY TOTAL VALUE [CURRENCY] [TAX AMOUNT [CURRENCY]]` |
| CAPRETURN     | `YYYY-MM-DD CAPRETURN TICKER QUANTITY TOTAL VALUE [CURRENCY] [FEES AMOUNT [CURRENCY]]`   |
| SPLIT/UNSPLIT | `YYYY-MM-DD SPLIT\|UNSPLIT TICKER RATIO VALUE`                                           |

- FEES, TAX default to 0 when omitted.
- Currency codes are optional ISO 4217; default is GBP.

### Comments

Lines starting with `#` are ignored. Inline `# comments` after data are stripped.

### Ticker Normalization

Ticker symbols are normalized to uppercase (`aapl` → `AAPL`).

### Error Messages

Parse failures report line number, problematic value, and expected format.

---

## Broker Conversion

### Converter Trait

Broker modules implement a converter that accepts file contents as strings (no filesystem IO) and returns CGT DSL. WASM-compatible.

### Schwab Converter

Parses Charles Schwab JSON exports (`BrokerageTransactions` array).

**Supported actions:** Buy, Sell, Stock Plan Activity (RSU), Cash Dividend, NRA Withholding.

- Transactions are output in chronological order (oldest first), regardless of input order.
- Unknown actions are captured as comments in the output, not as errors.
- Wire Sent, Wire Received, Credit Interest, and similar cash movements are skipped.
- Skipped transaction count is included in a header comment.

### Schwab Awards Parser

Parses Schwab Equity Awards JSON to obtain Fair Market Value and vest dates for RSU events.

- Reads `VestFairMarketValue` with `VestDate`; falls back to `FairMarketValuePrice` and parent `Date`.
- Symbol lookup is case-insensitive.
- Non-vesting actions (Wire Transfer, Tax Withholding, Tax Reversal, Forced Disbursement) with empty details are accepted without error.
- Unknown actions with empty details fail with an error.

### RSU Vest Dates (CG14250, ERSM20192)

The RSU vest date (from the awards file) is the CGT acquisition date, not the settlement date from transactions.

- Same Day rule matches sales on vest date with vest-date acquisitions.
- B&B window is calculated from vest date, not settlement date.
- FMV 7-day lookback: if settlement date does not exactly match awards, search up to 7 days back. Returns both FMV and the matched vest date.

### Dividend + Withholding Merging

When a Cash Dividend and NRA Withholding occur on the same date for the same symbol, they are merged into a single `DIVIDEND <ticker> TOTAL <value> [<currency>] TAX <amount> [<currency>]` line (no quantity).

### Date Format Handling

Schwab dates: `MM/DD/YYYY`, `YYYY-MM-DD`, or `as of` notation. For `02/25/2021 as of 02/21/2021`, the "as of" date is used.

---

## Output Formats

### Shared Formatting Rules

- **Currency:** £ symbol, 2 decimal places, comma thousands separators (e.g., £1,234.00). Negative: -£100.00.
- **Dates:** DD/MM/YYYY.
- **Tax years:** YYYY/YY (e.g., 2023/24).
- **Foreign currency:** `£118.42 (150 USD)`.
- **Unit prices:** Full precision, trailing zeros stripped (e.g., £4.6702).
- **Rounding:** Midpoint-away-from-zero (£100.995 → £101.00).
- **Ordering:** Date ascending, then ticker ascending, using a shared canonical comparator.

### Report Sections

Both plain text and PDF output include: **Summary**, **Tax Year Details**, **Holdings**, **Transactions**.

### Summary Table

Per tax year: disposal count (grouped per CG51560), net gain, total gains (before losses), total losses, gross proceeds (SA108 Box 21), annual exemption, taxable gain.

- Disposals footnote: grouped per CG51560.
- Proceeds footnote: corresponds to SA108 Box 21 "Disposal proceeds".
- Gains/Losses footnote: net allowable per SA108 after matching rules.

### Proceeds Breakdown

For each disposal:

- With fees: `<qty> × £<unit_price> = £<gross>` then `£<gross> - £<fees> fees = £<net>`.
- Without fees: `<qty> × £<unit_price> = £<gross>` (fees line omitted).
- Same-day merge: weighted average unit price = total_gross_proceeds / total_quantity.

### Holdings Display

Lists remaining holdings with ticker, quantity, and average cost. Shows "NONE" if empty.

### PDF-Specific

- Generated using embedded Typst engine with bundled fonts (no external tools).
- Output to `--output` path or default to input filename with `.pdf` extension.
- Superscript footnote markers in summary table headers.
- PDF generation errors are owned by the PDF formatter crate, not core.

---

## Testing Strategy

### Golden-File Pattern

`.cgt` input → `.json` expected output (and `.txt` for plain text).

Tests are the source of truth. Never remove or modify tests without proving incorrectness.

### Key Edge Cases

| Case                              | Expected Behavior                                                      |
| --------------------------------- | ---------------------------------------------------------------------- |
| Multi-currency same-day           | Same-day matching applies; both transactions converted to GBP          |
| B&B boundary D+30                 | Matches (within 30 days)                                               |
| B&B boundary D+31                 | Does not match; falls back to S104                                     |
| Partial B&B + S104 fallback       | Sell 100, buy back 40 within 30 days → 40 B&B + 60 S104                |
| Same-day buy-sell-buy             | All same-day transactions aggregated; net position determines matching |
| Capital return exceeds basis      | Fails with error referencing TCGA92/S122 and CG57847                   |
| Sell exceeds holdings             | Fails with validation error; no partial output                         |
| Interleaved same-day reservation  | Aggregate date+ticker reservation; B&B uses post-reservation remainder |
| Split then same-day sell          | Split applied before matching; disposal uses post-split quantities     |
| Tax year boundary (5 Apr / 6 Apr) | Gains attributed to correct year                                       |

### Comprehensive Fixtures

- **`SyntheticComplex.cgt`**: 5 tax years (2020/21–2024/25), 3+ tickers (ACME USD, BETA USD, GAMA GBP), all matching rules, corporate actions, multi-currency, boundary dates. Golden files: `tests/json/SyntheticComplex.json`, `tests/plain/SyntheticComplex.txt`.
- **`RealisticMultiYear.cgt`**: 2–3 tax years, multiple tickers, same-day/B&B/S104 matches, corporate actions.

### Cross-Validation

`scripts/cross-validate.py` compares results against external calculators (cgtcalc by mattjgalloway, cgt-calc). Discrepancies > £1 per tax year are reported as failures. Files with unsupported operations (SPLIT, UNSPLIT, CAPRETURN) are skipped for the relevant calculator. Output is sorted with acquisitions before disposals on the same day for external calculator compatibility.

### Schwab Test Fixtures

Synthetic Schwab JSON fixtures (`tests/schwab/synthetic-awards.json`, `tests/schwab/synthetic-transactions.json`) exercise RSU vesting patterns: multi-award same-day vesting, sell-to-cover, FMV vs sale price differences.
