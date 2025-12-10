# Research: Multi-Currency FX Conversion

## Decisions

### Currency syntax and defaults

- **Decision**: Accept ISO 4217 currency codes per transaction line; GBP remains the default when no code is provided.
- **Rationale**: Preserves backward compatibility while enabling multi-currency inputs.
- **Alternatives considered**: Require explicit currency on every line (rejected: breaks existing files); allow symbols instead of codes (rejected: ambiguous and harder to parse).

### FX source priority and fallback

- **Decision**: Prefer user-provided monthly XML rates from the CLI `--fx-folder`; if missing for a currency/month, fall back to bundled latest rates. If neither exists, fail that transaction with a clear message naming the missing currency/month.
- **Rationale**: Ensures up-to-date rates when supplied, but keeps the tool usable with bundled data and surfaces gaps explicitly.
- **Alternatives considered**: Fail immediately when any provided file is missing a month (rejected: too rigid); silently skip missing rates (rejected: unsafe).

### Rate selection window

- **Decision**: Select the monthly rate based on the transaction’s calendar month and currency; apply that rate to all amounts in that transaction.
- **Rationale**: Aligns with HMRC monthly published rates and keeps calculations predictable.
- **Alternatives considered**: Daily interpolation (rejected: not supported by source data); nearest available month (rejected: could mask missing data).

### Precision and rounding

- **Decision**: Use 6 decimal places internally for FX conversion; display GBP to 2 decimal places and original currencies using their standard minor units (e.g., JPY 0dp, USD/EUR 2dp).
- **Rationale**: Minimizes conversion drift while keeping reports readable and consistent with currency norms.
- **Alternatives considered**: 4dp internal (rejected: higher drift risk); uniform 2dp display for all currencies (rejected: misrepresents currencies like JPY).

### XML format handling

- **Decision**: Parse trade-tariff monthly rate XML files; expect per-currency monthly rates with effective month/year and a numeric rate value. Normalize currency codes to uppercase and trim whitespace.
- **Rationale**: Matches the source site format; normalization avoids parsing surprises from casing/spacing.
- **Alternatives considered**: Convert XML to CSV before use (rejected: extra step and format drift risk).

### Data structures for FX lookup

- **Decision**: Cache rates in memory using a hash map keyed by (currency_code, year, month) to allow O(1) lookups during batch processing; store source metadata with each entry.
- **Rationale**: Supports the performance target (10k transactions \<2 minutes) and avoids repeated XML reads.
- **Alternatives considered**: Per-lookup XML parsing (rejected: slow); list scans (rejected: O(n) per lookup).

### Bundled rates versioning

- **Decision**: Ship a latest-rates XML snapshot with a recorded effective month/year; log when bundled rates are used.
- **Rationale**: Ensures the tool works offline and provides traceability when bundled data is applied.
- **Alternatives considered**: Require online fetch (rejected: networkless requirement); omit bundled rates (rejected: harms usability).

## Open Points

- None; all clarifications resolved.
