# Data Model: Multi-Currency FX Conversion

## Entities

### Currency Amount

- **Fields**: `amount` (decimal), `currency` (ISO 4217 code, default GBP), `display_minor_units` (per currency standard), `source_line` (original text for audit).
- **Rules**: Currency code uppercased; if absent, treat as GBP. Stored both as original amount/currency and as converted GBP amount for calculations.

### Monthly FX Rate

- **Fields**: `currency` (ISO 4217), `year` (YYYY), `month` (1-12), `rate_to_gbp` (decimal), `source` (Provided|Bundled), `source_path` (optional), `loaded_at` (timestamp).
- **Rules**: One entry per currency-month. If duplicate sources exist, prefer latest file timestamp. Missing month for a needed currency is an error.

### FX Rate Cache

- **Fields**: `rates` (map keyed by (currency, year, month) → Monthly FX Rate), `default_source` (bundled snapshot metadata), `folder_source` (path, if provided).
- **Rules**: Populated once per run; lookups are O(1). Must record which source was used for each conversion for audit/logging.

### Conversion Result

- **Fields**: `original_amount` (Currency Amount), `converted_gbp` (decimal, 6dp internal), `used_rate` (Monthly FX Rate), `display_original` (formatted with minor units), `display_gbp` (2dp).
- **Rules**: Calculations always in GBP; display original alongside GBP. Internal rounding at 6dp; display rounding per currency minor units.

## Relationships

- Currency Amount uses Monthly FX Rate (by currency + year/month) to produce Conversion Result.
- FX Rate Cache aggregates Monthly FX Rates from provided folder and bundled snapshot.

## Lifecycle

- Load: Parse XML files (provided folder, then bundled) into FX Rate Cache.
- Convert: For each transaction, resolve currency/month → rate → Conversion Result.
- Report: Render GBP primary amounts and original amounts in parentheses; include source info optionally in logs.

## Volume & Scale Assumptions

- Up to ~10k transactions per run (per success criteria) with mixed currencies.
- FX table size small (dozens of currencies × months) — fits comfortably in memory.
