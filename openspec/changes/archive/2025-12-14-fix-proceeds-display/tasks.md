# Tasks: Fix Proceeds Display in Plain Formatter

## 1. Spec Updates

- [x] Add plain-formatter requirement for disposal proceeds breakdown that includes sale expenses, omits the `- £0` term when expenses are zero, and matches the computed net proceeds using currency minor units.
- [x] Add pdf-formatter requirement enforcing the same proceeds breakdown and currency-precision policy for parity with plain text.

## 2. Implementation (post-approval)

- [x] Update plain text formatter disposal calculation line to render `quantity × unit price - sale expenses = net proceeds`, omitting the `- £0` term when expenses are zero, and format proceeds using each currency’s minor units; ensure summary/holdings and other outputs respect the same precision policy.
- [x] Update the PDF formatter to use the same proceeds breakdown and precision rules for parity.

## 3. Fixtures & Tests

- [x] Refresh plain golden fixtures affected by proceeds formatting (e.g., MultipleMatches; audit other sells with fees).
- [x] Extend formatter/CLI tests to cover proceeds lines with sale expenses and assert the displayed math matches the computed proceeds.

## 4. Validation

- [x] Run targeted formatter checks (e.g., `cargo test -p cgt-cli test_plain_format_outputs`) and rerun broader suites if needed after fixture updates.
