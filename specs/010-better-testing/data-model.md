# Data Model: Better Testing Coverage

## Entities

### CGT Test Fixture

- **Purpose**: Encapsulate a set of CGT transactions for regression tests.
- **Fields**:
  - `date` (DD/MM/YYYY)
  - `asset` (string ticker/name)
  - `action` (BUY, SELL, DIVIDEND, CAPRETURN, etc.)
  - `quantity` (decimal; may be zero for CAPRETURN if not provided)
  - `price` (decimal; per-unit for buys/sells; dividend per-unit amount)
  - `expenses` (decimal; fees/stamp duty; may be zero)
  - `currency` (GBP expected; non-GBP should be rejected/flagged)
- **Constraints/Validation**:
  - Dates must be valid and map to a tax year; 2024/25 disposals split at 30/10/2024 for rate testing.
  - DIVIDEND lines accepted even if holdings are zero; they should not mutate pools when none held.
  - CAPRETURN reduces Section 104 pool cost by lump sum regardless of quantity presence.
  - Expenses increase allowable costs; proceeds exclude expenses.
  - Non-GBP amounts produce a guardrail failure (unsupported).

### Tax Period Banding

- **Purpose**: Represent mapping from transaction dates to applicable CGT rate bands for assertions.
- **Attributes**:
  - `tax_year_start` (u16; e.g., 2024 for 2024/25)
  - `segment` (pre-cutover, post-cutover)
  - `cutover_date` (30/10/2024 for main rate change)
  - `rate_main_basic` (pre: 10%, post: 18%)
  - `rate_main_higher` (pre: 20%, post: 24%)
  - `rate_residential_basic` (18%)
  - `rate_residential_higher` (24%)
- **Constraints**: Residential rates unchanged across cutover; main rates change on/after cutover.

### Report Assertion Target (Text Output)

- **Purpose**: Expected values extracted from generated text reports to compare against computed expectations.
- **Fields**:
  - `matched_quantity` (for B&B and Section 104 narratives)
  - `proceeds_reported`
  - `allowable_cost_reported`
  - `gains_losses_reported`
  - `messages` (e.g., unsupported currency warnings)
- **Constraints**: Values must align with computed expectations; rounding only at final report stage.

## Relationships

- A **CGT Test Fixture** maps to one or more **Tax Period Banding** segments based on dates.
- A **CGT Test Fixture** yields a **Report Assertion Target** derived from the text output for comparison in tests.

## State & Transitions

- Section 104 pool state evolves per transaction order; CAPRETURN reduces pool cost on its date; accumulation dividends adjust pool only when holdings exist on that date.

## Volumes & Scale Assumptions

- Fixtures remain small (tens of transactions) to keep test runtime low; no large-scale performance testing required.
