# Research: Better Testing Coverage

## Decisions

### 2024/25 rate split (pre/post 30 Oct 2024)

- **Decision**: Apply old main CGT rates to disposals before 30 Oct 2024 and 18%/24% main rates on/after 30 Oct 2024; residential rates unchanged.
- **Rationale**: HMRC policy paper “Capital Gains Tax — rates of tax” (updated 6 Nov 2024) mandates intra-year split for 2024/25.
- **Alternatives considered**: Treat 2024/25 as uniform rates for entire tax year (rejected; non-compliant).

### Accumulation dividends handling

- **Decision**: Adjust Section 104 pool only for units held on dividend date; do not retroactively alter gains for disposed units; allow dividends after full disposal without error (no pool change).
- **Rationale**: HMRC accumulation treatment ties to holdings on ex-dividend date; prevents misstated gains on prior disposals.
- **Alternatives considered**: Pro-rata dividends across historical disposals (rejected; incorrect), hard error when holdings are zero (rejected; invalid UX).

### CAPRETURN equalisation

- **Decision**: Treat CAPRETURN as lump-sum capital return that reduces pool cost on payment date; no share count required.
- **Rationale**: Equalisation is a one-off capital return; simplifying input matches HMRC guidance and issue #15 feedback.
- **Alternatives considered**: Require per-share equalisation (rejected; overconstrained and inaccurate).

### Expenses and rounding

- **Decision**: Fees and stamp duty increase allowable costs (buys/sells) and are excluded from proceeds; maintain high precision internally, rounding only at final presentation.
- **Rationale**: HMRC rules on allowable costs; avoids rounding drift highlighted in issue #12.
- **Alternatives considered**: Per-transaction rounding or including fees in proceeds (rejected; inconsistent with guidance and leads to drift).

### FX handling

- **Decision**: Until FX support exists, reject/flag non-GBP amounts explicitly in tests.
- **Rationale**: Accuracy cannot be guaranteed without conversion; guardrail prevents silent miscalculation.
- **Alternatives considered**: Implicitly treat as GBP (rejected; unsafe), mocked FX rates (rejected; misleading).
- **Implementation Note**: The current DSL does not support currency codes. All monetary values are raw decimals implicitly assumed to be GBP. Attempting to use currency codes (e.g., `USD 150`) would result in a parse error ("expected decimal number"). No explicit FX guardrail test fixture is needed since the parser inherently rejects non-numeric price values.

### Reporting scope

- **Decision**: Focus assertions on plain-text output; cross-format (PDF) parity excluded from this effort.
- **Rationale**: User directive "no need for cross-format tests"; keeps scope on primary report.
- **Alternatives considered**: Maintain dual text/PDF assertions (rejected; out of scope per user).

---

## Worked Examples (for fixture verification)

### 2024/25 Rate Split Calculations (T003)

**HMRC Rule**: For tax year 2024/25, CGT main rates changed on 30 October 2024:

- **Before 30 Oct 2024**: Basic rate 10%, Higher rate 20%
- **On/after 30 Oct 2024**: Basic rate 18%, Higher rate 24%

Note: The calculator computes gains/losses; rate application happens at tax return time.
The fixture tests that disposals are correctly dated and grouped by pre/post cutover.

**Worked Example - RateSplit2024 fixture**:

```
Disposal 1: 29 Oct 2024 (pre-cutover)
  Buy: 100 shares @ £50 + £10 expenses = £5,010 cost
  Sell same day: 100 @ £60 - £10 expenses = £5,990 proceeds
  Gain: £5,990 - £5,010 = £980

Disposal 2: 30 Oct 2024 (post-cutover)
  Buy: 100 shares @ £50 + £10 expenses = £5,010 cost
  Sell same day: 100 @ £70 - £10 expenses = £6,990 proceeds
  Gain: £6,990 - £5,010 = £1,980

Total gain: £980 + £1,980 = £2,960
Total loss: £0
Net gain: £2,960
```

### Accumulation Dividend Pool Adjustments (T004)

**HMRC Rule**: Accumulation dividends increase the Section 104 pool cost basis only for units held on the dividend date. If a partial disposal occurred before the dividend, the dividend adjustment applies only to remaining units.

**Worked Example - AccumulationDividend fixture**:

```
Step 1: Buy 100 shares @ £10 + £5 expenses
  Pool: 100 shares, cost = £1,005

Step 2: Sell 50 shares @ £12 - £5 expenses (S104 match)
  Proceeds: £595
  Cost: 50/100 × £1,005 = £502.50
  Gain: £595 - £502.50 = £92.50
  Pool after: 50 shares, cost = £502.50

Step 3: Dividend on remaining 50 shares, TOTAL £25
  Pool adjustment: cost = £502.50 + £25 = £527.50
  (Only adjusts for 50 shares held on dividend date)

Step 4: Sell remaining 50 shares @ £11 - £5 expenses
  Proceeds: £545
  Cost: £527.50
  Gain: £545 - £527.50 = £17.50

Total gains: £92.50 + £17.50 = £110
```

### CAPRETURN Pool Cost Reduction (T005)

**HMRC Rule**: Capital return (equalisation payment) reduces the Section 104 pool cost basis by the total amount received. This is a lump-sum reduction, not per-share.

**Worked Example - CapReturnEqualisation fixture**:

```
Step 1: Buy 100 shares @ £20 + £10 expenses
  Pool: 100 shares, cost = £2,010

Step 2: CAPRETURN payment, TOTAL £100 (no quantity needed)
  Pool: 100 shares, cost = £2,010 - £100 = £1,910

Step 3: Sell 100 shares @ £25 - £10 expenses
  Proceeds: £2,490
  Cost: £1,910
  Gain: £2,490 - £1,910 = £580

Total gain: £580
```

### Expenses and Stamp Duty Treatment (T006)

**HMRC Rule**:

- Buy expenses (fees, stamp duty): Added to allowable cost
- Sell expenses (fees): Deducted from proceeds

No rounding until final report presentation; internal calculations use full precision.

**Worked Example - ExpensesRounding fixture**:

```
Step 1: Buy 150 shares @ £33.33 + £12.50 stamp + £9.99 commission
  Cost: (150 × £33.33) + £12.50 + £9.99 = £4,999.50 + £22.49 = £5,021.99

Step 2: Sell 150 shares @ £40.00 - £9.99 commission
  Proceeds: (150 × £40.00) - £9.99 = £6,000 - £9.99 = £5,990.01

Gain: £5,990.01 - £5,021.99 = £968.02

The precision is maintained internally; report shows rounded values.
```
