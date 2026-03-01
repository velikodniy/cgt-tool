//! MCP resource definitions and string constants.

// Error message hints (used by server.rs for helpful error messages)

/// Hint for unknown action type errors.
pub const HINT_UNKNOWN_ACTION: &str =
    "HINT: Valid actions are: BUY, SELL, DIVIDEND, ACCUMULATION, CAPRETURN, SPLIT, UNSPLIT";

/// Hint for invalid currency code errors.
pub const HINT_INVALID_CURRENCY: &str = "HINT: Use ISO 4217 currency codes like GBP, USD, EUR.\nUse get_fx_rate tool to check available rates.";

/// Hint for missing FX rate errors.
pub const HINT_MISSING_FX_RATE: &str = r#"HINT: This transaction uses a foreign currency but no exchange rate is available.
Use the get_fx_rate tool to check available rates.
Ensure the transaction date falls within a period with published HMRC rates."#;

/// Hint for selling shares without prior purchase.
pub const HINT_SELL_WITHOUT_BUY: &str = r#"HINT: You're trying to sell shares you don't own.
Check that:
1. BUY transactions exist before the SELL
2. The ticker symbols match exactly (case-insensitive)
3. The sell quantity doesn't exceed owned shares"#;

/// Hint for invalid transaction data.
pub const HINT_INVALID_TRANSACTION: &str = r#"HINT: Check transaction data for:
- Negative quantities or prices
- Zero split ratios
- Invalid dates"#;

/// Hint for invalid date format.
pub const HINT_DATE_FORMAT: &str = r#"Expected format: YYYY-MM-DD (e.g., 2024-06-15)

Common mistakes:
- DD/MM/YYYY → use YYYY-MM-DD
- MM/DD/YYYY → use YYYY-MM-DD
- Missing leading zeros → 2024-1-5 should be 2024-01-05"#;

/// Hint for currency that exists but rate not available.
pub const HINT_FX_RATE_EXISTS: &str = r#"This currency exists but rate is not available for this period.

HMRC rates are available from January 2015 onwards.
Recent months may not yet have published rates."#;

/// Hint for unknown currency code.
pub const HINT_FX_RATE_UNKNOWN: &str = r#"Currency may not be a valid ISO 4217 code.
Common currency codes: USD, EUR, JPY, CHF, AUD, CAD, CNY

Check https://en.wikipedia.org/wiki/ISO_4217 for valid codes."#;

/// Quick DSL syntax reference for error messages.
pub const DSL_SYNTAX_REFERENCE: &str = r#"DSL Syntax Reference:
  BUY:          YYYY-MM-DD BUY TICKER QUANTITY @ PRICE [CURRENCY] [FEES AMOUNT]
  SELL:         YYYY-MM-DD SELL TICKER QUANTITY @ PRICE [CURRENCY] [FEES AMOUNT]
  DIVIDEND:     YYYY-MM-DD DIVIDEND TICKER TOTAL VALUE [TAX AMOUNT]
  ACCUMULATION: YYYY-MM-DD ACCUMULATION TICKER QUANTITY TOTAL VALUE [TAX AMOUNT]
  SPLIT:        YYYY-MM-DD SPLIT TICKER RATIO VALUE

Example: 2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD

Note: FEES and TAX clauses are optional (default to 0 when omitted).

For JSON format, start input with '[' character."#;

/// Example transaction JSON for error messages.
pub const EXAMPLE_TRANSACTION: &str =
    r#"{"date":"2024-01-15","ticker":"AAPL","action":"BUY","amount":"100","price":"150"}"#;

// MCP Resources

/// An MCP resource definition.
pub struct Resource {
    /// Unique URI identifier for the resource.
    pub uri: &'static str,
    /// Display name for the resource.
    pub name: &'static str,
    /// Content of the resource (static text).
    pub content: &'static str,
}

/// DSL syntax documentation resource.
pub static DSL_SYNTAX: Resource = Resource {
    uri: "cgt://docs/dsl-syntax",
    name: "CGT-Tool CLI DSL Syntax - text format for .cgt files used by cgt-tool CLI",
    content: r#"# CGT DSL Syntax Reference

This DSL is used by the cgt-tool CLI (https://github.com/velikodniy/cgt-tool) to define share transactions in `.cgt` text files.

Use this syntax when generating transaction files for the CLI tool.

## Syntax Conventions

- **UPPERCASE** = keywords (type exactly as shown)
- `<angle brackets>` = placeholders (replace with your values)
- `[square brackets]` = optional parts
- `|` = alternatives (choose one)

## Date Format

Dates must be in ISO format: `YYYY-MM-DD`

Example: `2024-03-15`

## Commands

### BUY
Acquire shares.

```
<date> BUY <ticker> <quantity> @ <price> [<currency>] [FEES <amount> [<currency>]]
```

Examples:
```
2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD
2024-01-15 BUY VWRL 50 @ 85.50 GBP FEES 5 GBP
2024-01-15 BUY TSLA 10 @ 200  # No currency = GBP, no fees
```

### SELL
Dispose of shares.

```
<date> SELL <ticker> <quantity> @ <price> [<currency>] [FEES <amount> [<currency>]]
```

Examples:
```
2024-06-20 SELL AAPL 50 @ 180 USD FEES 10 USD
2024-06-20 SELL VWRL 25 @ 90 GBP
```

### DIVIDEND
Record an ordinary cash dividend payment. No quantity needed; does not affect cost basis.

```
<date> DIVIDEND <ticker> TOTAL <amount> [<currency>] [TAX <amount> [<currency>]]
```

Examples:
```
2024-03-01 DIVIDEND VWRL TOTAL 50 GBP           # TAX defaults to 0
2024-03-01 DIVIDEND VWRL TOTAL 50 GBP TAX 5 GBP # With tax withheld
```

### ACCUMULATION
Record a dividend reinvestment in an accumulation fund. Increases cost basis by the dividend amount (notional disposal and reacquisition at the same price).

```
<date> ACCUMULATION <ticker> <quantity> TOTAL <amount> [<currency>] [TAX <amount> [<currency>]]
```

Examples:
```
2024-03-01 ACCUMULATION VWRL 100 TOTAL 50 GBP           # TAX defaults to 0
2024-03-01 ACCUMULATION VWRL 100 TOTAL 50 GBP TAX 5 GBP # With tax withheld
```

### CAPRETURN
Record capital return (reduces cost basis).

```
<date> CAPRETURN <ticker> <quantity> TOTAL <amount> [<currency>] [FEES <amount> [<currency>]]
```

Example:
```
2024-04-01 CAPRETURN AAPL 100 TOTAL 200 USD FEES 0 USD
```

### SPLIT
Stock split (increases quantity, same total cost).

```
<date> SPLIT <ticker> RATIO <ratio>
```

Example (2-for-1 split):
```
2024-02-01 SPLIT AAPL RATIO 2
```

### UNSPLIT
Reverse stock split (decreases quantity, same total cost).

```
<date> UNSPLIT <ticker> RATIO <ratio>
```

Example (1-for-10 reverse split):
```
2024-02-01 UNSPLIT AAPL RATIO 10
```

## Currency Codes

Common currency codes:
- `GBP` - British Pound (default if omitted)
- `USD` - US Dollar
- `EUR` - Euro

Foreign currencies are converted to GBP using HMRC monthly average rates.

## Comments

Lines starting with `#` are comments and ignored:

```
# This is a comment
2024-01-15 BUY AAPL 100 @ 150 USD  # Inline comment
```

## Complete Example

```
# Portfolio transactions for 2024

# Initial purchases
2024-01-10 BUY VWRL 100 @ 85 GBP FEES 5 GBP
2024-01-15 BUY AAPL 50 @ 150 USD FEES 10 USD

# Cash dividend (no cost basis change)
2024-03-15 DIVIDEND VWRL TOTAL 25 GBP

# Accumulation fund dividend reinvestment (increases cost basis)
2024-03-15 ACCUMULATION VWRA 100 TOTAL 30 GBP

# Apple stock split
2024-04-01 SPLIT AAPL RATIO 2

# Partial sale
2024-06-01 SELL AAPL 25 @ 180 USD FEES 10 USD

# Year-end rebalance
2024-12-15 SELL VWRL 50 @ 92 GBP FEES 5 GBP
```
"#,
};

/// HMRC tax rules documentation resource.
pub static TAX_RULES: Resource = Resource {
    uri: "cgt://docs/tax-rules",
    name: "UK HMRC Share Matching Rules - Same Day, Bed & Breakfast, Section 104 Pool explained",
    content: include_str!("../../../docs/tax-rules.md"),
};

/// All available resources.
pub static RESOURCES: &[&Resource] = &[&DSL_SYNTAX, &TAX_RULES];

/// Server instructions shown on MCP initialization.
pub const SERVER_INSTRUCTIONS: &str = r#"UK Capital Gains Tax calculator implementing HMRC share matching rules. All results are in GBP.

## IMPORTANT: Currency Handling

By default, all amounts are treated as GBP. For US stocks or other foreign stocks, you MUST specify the currency explicitly!

Price format examples:
  GBP (default): "price": "150"
  USD explicit:  "price": {"amount": "150", "currency": "USD"}
  EUR explicit:  "price": {"amount": "150", "currency": "EUR"}

Common US stocks (AAPL, MSFT, GOOGL, AMZN, TSLA, META, NVDA) are priced in USD.

Supported currencies: GBP, USD, EUR, JPY, CHF, AUD, CAD, CNY, and other ISO 4217 codes.
Use get_fx_rate to check available rates for a currency/date.

## Tax Year

The year parameter is the START year. Use 2024 for any disposals between 6 April 2024 and 5 April 2025.

## Transaction Actions (case-insensitive)

- BUY/SELL: Acquire or dispose of shares (triggers CGT on SELL)
- SPLIT: Stock split (e.g., 2-for-1). Use ratio: "2"
- UNSPLIT: Reverse split (e.g., 1-for-10). Use ratio: "10"
- DIVIDEND: Ordinary cash dividend (no quantity, no cost basis change)
- ACCUMULATION: Accumulation fund dividend reinvestment (has quantity, increases cost basis)
- CAPRETURN: Capital return (reduces cost basis)

## Transaction Format

Required: date, ticker, action, amount (quantity), price (per share)
Optional: fees (defaults to 0)

Tickers and actions are case-insensitive (aapl = AAPL, buy = BUY).
Amounts must be positive numbers.

## Examples

US Stock buy (AAPL in USD):
{"date":"2024-01-15","ticker":"AAPL","action":"BUY","amount":"100","price":{"amount":"185","currency":"USD"},"fees":{"amount":"10","currency":"USD"}}

UK Stock buy (VOD in GBP):
{"date":"2024-01-15","ticker":"VOD","action":"BUY","amount":"100","price":"120"}

Stock split (2-for-1):
{"date":"2024-06-01","ticker":"AAPL","action":"SPLIT","ratio":"2"}

Stock unsplit/reverse split (1-for-10):
{"date":"2024-06-01","ticker":"XYZ","action":"UNSPLIT","ratio":"10"}

Cash dividend (no quantity):
{"date":"2024-03-01","ticker":"VWRL","action":"DIVIDEND","total_value":"50"}

Accumulation fund dividend reinvestment (has quantity):
{"date":"2024-03-01","ticker":"VWRL","action":"ACCUMULATION","amount":"100","total_value":"50"}

## Matching Rules (in priority order)

1. Same Day: Sells matched to same-day buys first
2. Bed & Breakfast: Then to buys within 30 days AFTER the sale
3. Section 104 Pool: Finally to the average cost pool

## Available Tools

- parse_transactions: Validate and parse transactions (see tool description for JSON schema)
- calculate_report: Calculate CGT for a tax year
- explain_matching: Explain how a sale was matched to acquisitions
- get_fx_rate: Get HMRC monthly FX rates
- convert_to_dsl: Convert JSON to DSL format for cgt-tool CLI

## Resources (require manual addition in Claude Desktop)

Resources must be explicitly added in Claude Desktop settings to be available.
- cgt://docs/dsl-syntax - DSL syntax for .cgt files
- cgt://docs/tax-rules - Detailed HMRC share matching rules"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_resources_have_valid_uris() {
        for resource in RESOURCES {
            assert!(
                resource.uri.contains("://"),
                "Resource '{}' should have a valid URI scheme",
                resource.name
            );
        }
    }

    #[test]
    fn test_all_resources_have_unique_uris() {
        let uris: Vec<_> = RESOURCES.iter().map(|r| r.uri).collect();
        for (i, uri) in uris.iter().enumerate() {
            for (j, other) in uris.iter().enumerate() {
                if i != j {
                    assert_ne!(uri, other, "Resource URIs must be unique");
                }
            }
        }
    }

    #[test]
    fn test_all_resources_have_content() {
        for resource in RESOURCES {
            assert!(
                !resource.content.is_empty(),
                "Resource '{}' should have content",
                resource.name
            );
        }
    }

    #[test]
    fn test_dsl_syntax_contains_commands() {
        assert!(DSL_SYNTAX.content.contains("BUY"));
        assert!(DSL_SYNTAX.content.contains("SELL"));
    }

    #[test]
    fn test_tax_rules_contains_matching_rules() {
        assert!(TAX_RULES.content.contains("Same Day"));
        assert!(TAX_RULES.content.contains("Bed & Breakfast"));
        assert!(TAX_RULES.content.contains("Section 104"));
    }
}
