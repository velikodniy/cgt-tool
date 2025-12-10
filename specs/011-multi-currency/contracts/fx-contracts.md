# Contracts: FX Rates and CLI Input

## CLI Argument Contract

- **Flag**: `--fx-folder <path>` (exact spelling to be finalized in implementation; must accept absolute or relative paths).
- **Behavior**:
  - If provided: load monthly FX XML files from the folder; use these rates preferentially.
  - If not provided: use bundled latest rates.
  - If a required currency/month is missing in provided files: fall back to bundled; if still missing, emit a clear error naming the currency and month.
- **Validation**: Fail fast on unreadable folder, malformed XML, or unsupported currency codes (list supported codes in help).

## XML Rate File Contract

- **Source**: Files downloaded from https://www.trade-tariff.service.gov.uk/exchange_rates
- **Expected contents (per currency per month)**:
  - `currency_code` (ISO 4217, uppercase)
  - `year` (YYYY)
  - `month` (1-12)
  - `rate_to_gbp` (decimal)
  - Optional metadata: publication date, source URL
- **Rules**:
  - Normalize currency codes to uppercase and trim whitespace.
  - If multiple files provide the same currency/month, prefer the newest by file timestamp; log duplicates.
  - Missing required currency/month after bundled fallback is an error.

## Reporting Contract

- **Calculations**: Always in GBP using the monthly rate for the transaction’s month/currency.
- **Display**: GBP as primary; original amount and currency in parentheses for non-GBP lines.
- **Precision**: 6dp internal conversions; display GBP to 2dp and original currency in its standard minor units.
