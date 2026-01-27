//! MCP server implementation.

use crate::McpServerError;
use crate::resources::{
    DSL_SYNTAX_REFERENCE, EXAMPLE_TRANSACTION, HINT_DATE_FORMAT, HINT_FX_RATE_EXISTS,
    HINT_FX_RATE_UNKNOWN, HINT_INVALID_CURRENCY, HINT_INVALID_TRANSACTION, HINT_MISSING_FX_RATE,
    HINT_SELL_WITHOUT_BUY, HINT_UNKNOWN_ACTION, RESOURCES, SERVER_INSTRUCTIONS,
};
use cgt_core::calculator::calculate;
use cgt_core::parser::parse_file;
use cgt_core::{CurrencyAmount, Disposal, MatchRule, TaxReport, Transaction};
use cgt_money::{FxCache, load_default_cache};
use chrono::Datelike;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::schemars::{self, JsonSchema};
use rmcp::service::{RequestContext, RoleServer, ServiceExt};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Request parameters for parse_transactions tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParseTransactionsRequest {
    /// Transactions as JSON array or CGT DSL text.
    pub transactions: String,
}

/// Request parameters for calculate_report tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateReportRequest {
    /// Transactions as JSON array or CGT DSL text.
    pub transactions: String,
    /// Tax year start (e.g., 2024 for tax year 2024/25). If omitted, report includes all years.
    pub year: Option<i32>,
}

/// Request parameters for explain_matching tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainMatchingRequest {
    /// Transactions as JSON array or CGT DSL text.
    pub transactions: String,
    /// Date of disposal in YYYY-MM-DD format.
    pub disposal_date: String,
    /// Ticker symbol of the asset.
    pub ticker: String,
}

/// Request parameters for get_fx_rate tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFxRateRequest {
    /// Currency code (e.g., "USD", "EUR").
    pub currency: String,
    /// Year (e.g., 2024).
    pub year: i32,
    /// Month (1-12).
    pub month: u32,
}

/// Request parameters for convert_to_dsl tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConvertToDslRequest {
    /// Transactions as JSON array.
    pub transactions: String,
}

/// Response for get_fx_rate tool.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FxRateResponse {
    /// Exchange rate (units of foreign currency per GBP).
    pub rate: String,
    /// Currency code.
    pub currency: String,
    /// Period in YYYY-MM format.
    pub period: String,
}

/// Explanation of how a disposal was matched.
#[derive(Debug, Serialize, JsonSchema)]
pub struct MatchExplanation {
    /// Date of the disposal.
    pub disposal_date: String,
    /// Ticker symbol.
    pub ticker: String,
    /// Total quantity disposed.
    pub quantity: String,
    /// Total proceeds from disposal.
    pub proceeds: String,
    /// Individual matches explaining how the disposal was matched.
    pub matches: Vec<MatchDetail>,
    /// Total gain or loss from this disposal.
    pub total_gain_or_loss: String,
}

/// Detail of a single match within a disposal.
#[derive(Debug, Serialize, JsonSchema)]
pub struct MatchDetail {
    /// Matching rule applied.
    pub rule: String,
    /// Quantity matched by this rule.
    pub quantity: String,
    /// Allowable cost for this match.
    pub allowable_cost: String,
    /// Gain or loss from this match.
    pub gain_or_loss: String,
    /// Acquisition date if applicable (for B&B matches).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisition_date: Option<String>,
    /// Human-readable explanation.
    pub explanation: String,
}

/// CGT MCP Server that handles tool and resource requests.
#[derive(Clone)]
pub struct CgtServer {
    fx_cache: Option<FxCache>,
    tool_router: ToolRouter<Self>,
}

impl CgtServer {
    /// Create a new CGT MCP server.
    pub fn new() -> Result<Self, McpServerError> {
        let fx_cache = load_default_cache()?;
        Ok(Self {
            fx_cache: Some(fx_cache),
            tool_router: Self::tool_router(),
        })
    }

    /// Parse transactions from JSON array or CGT DSL text.
    fn parse_input(&self, input: &str) -> Result<Vec<Transaction>, McpError> {
        let trimmed = input.trim();

        // Try JSON first (starts with '[')
        if trimmed.starts_with('[') {
            return serde_json::from_str(trimmed)
                .map_err(|e| Self::format_json_parse_error(e, trimmed));
        }

        // Fall back to DSL parsing
        self.parse_dsl(trimmed)
    }

    /// Format a JSON parse error with helpful context.
    fn format_json_parse_error(e: serde_json::Error, input: &str) -> McpError {
        let line = e.line();
        let column = e.column();
        let error_msg = e.to_string();
        let line_content = input.lines().nth(line.saturating_sub(1)).unwrap_or("");
        let pointer = " ".repeat(column.saturating_sub(1));

        let hint = if error_msg.contains("missing field") {
            let field = error_msg.split('`').nth(1).unwrap_or("unknown");
            format!(
                r#"HINT: The '{field}' field is required.

Required fields for BUY/SELL:
- date: "YYYY-MM-DD"
- ticker: "SYMBOL"
- action: "BUY" or "SELL"
- amount: "quantity" (number of shares)
- price: "amount" or {{"amount": "X", "currency": "USD"}}

Optional: fees (defaults to 0)"#
            )
        } else if error_msg.contains("invalid type") {
            r#"HINT: Check that numeric values are strings (e.g., "100" not 100).
Price/fees can be:
- A string for GBP: "150.00"
- An object for any currency: {"amount": "150.00", "currency": "USD"}"#
                .to_string()
        } else if error_msg.contains("unknown variant") {
            HINT_UNKNOWN_ACTION.to_string()
        } else if error_msg.contains("invalid currency") || error_msg.contains("currency code") {
            HINT_INVALID_CURRENCY.to_string()
        } else {
            String::new()
        };

        let msg = format!(
            "JSON parse error at line {line}, column {column}:\n\n\
             {line_content}\n\
             {pointer}^\n\n\
             Error: {error_msg}\n\n\
             {hint}\n\n\
             Example valid transaction:\n\
             {EXAMPLE_TRANSACTION}"
        );

        McpError::invalid_params(msg, None)
    }

    /// Parse CGT DSL content and return transactions.
    fn parse_dsl(&self, content: &str) -> Result<Vec<Transaction>, McpError> {
        parse_file(content).map_err(|e| {
            let msg = format!("DSL Parse Error:\n\n{e}\n\n{DSL_SYNTAX_REFERENCE}");
            McpError::invalid_params(msg, None)
        })
    }

    /// Calculate report for a tax year (or all years if None).
    fn do_calculate_report(&self, input: &str, year: Option<i32>) -> Result<TaxReport, McpError> {
        let transactions = self.parse_input(input)?;

        if transactions.is_empty() {
            return Err(McpError::invalid_params(
                "No transactions provided. Please provide at least one BUY or SELL transaction.",
                None,
            ));
        }

        calculate(transactions, year, self.fx_cache.as_ref())
            .map_err(|e| Self::format_calculation_error(e, year))
    }

    /// Format a calculation error with helpful context.
    fn format_calculation_error(e: cgt_core::CgtError, year: Option<i32>) -> McpError {
        let error_str = e.to_string();

        let hint = if error_str.contains("Missing FX rate") {
            HINT_MISSING_FX_RATE
        } else if error_str.contains("no prior acquisitions")
            || error_str.contains("exceeds holding")
        {
            HINT_SELL_WITHOUT_BUY
        } else if error_str.contains("Invalid transaction") {
            HINT_INVALID_TRANSACTION
        } else {
            ""
        };

        let year_context = match year {
            Some(y) => {
                let year_end = (y + 1) % 100;
                format!(
                    "Calculation Error for tax year {y}/{year_end:02}:\n\n\
                     {error_str}\n\n\
                     {hint}\n\n\
                     UK Tax Year {y}/{year_end:02} runs from 6 April {y} to 5 April {}.",
                    y + 1
                )
            }
            None => {
                format!(
                    "Calculation Error:\n\n\
                     {error_str}\n\n\
                     {hint}"
                )
            }
        };

        McpError::invalid_params(year_context, None)
    }

    /// Find a disposal by date and ticker.
    fn find_disposal(
        &self,
        report: &TaxReport,
        date_str: &str,
        ticker: &str,
    ) -> Result<Disposal, McpError> {
        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            let msg = format!("Invalid date format: '{date_str}'\n\n{HINT_DATE_FORMAT}");
            McpError::invalid_params(msg, None)
        })?;

        // Search in all tax years
        for year_summary in &report.tax_years {
            for disposal in &year_summary.disposals {
                if disposal.date == date && disposal.ticker.eq_ignore_ascii_case(ticker) {
                    return Ok(disposal.clone());
                }
            }
        }

        // Not found - provide helpful error with context
        let all_disposals: Vec<_> = report.tax_years.iter().flat_map(|y| &y.disposals).collect();

        let same_ticker: Vec<_> = all_disposals
            .iter()
            .filter(|d| d.ticker.eq_ignore_ascii_case(ticker))
            .collect();

        let all_tickers: Vec<_> = all_disposals
            .iter()
            .map(|d| d.ticker.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let detail = if same_ticker.is_empty() {
            if all_tickers.is_empty() {
                format!(
                    r#"No SELL transactions found for ticker '{ticker}'.

No disposals exist in the provided transactions.
Make sure you have SELL transactions in your data."#
                )
            } else {
                let tickers_list: String =
                    all_tickers.iter().map(|t| format!("  - {t}\n")).collect();
                format!(
                    r#"No SELL transactions found for ticker '{ticker}'.

Available tickers with disposals:
{tickers_list}
Did you mean one of these? Ticker matching is case-insensitive."#
                )
            }
        } else {
            let dates_list: String = same_ticker
                .iter()
                .map(|d| format!("  - {} (sold {} shares)\n", d.date, d.quantity))
                .collect();
            format!(
                r#"'{ticker}' has disposals on these dates:
{dates_list}
Use one of these dates in your query."#
            )
        };

        let msg = format!("No disposal found for '{ticker}' on {date_str}\n\n{detail}");
        Err(McpError::invalid_params(msg, None))
    }
}

#[tool_router]
impl CgtServer {
    #[tool(
        description = "Validate and parse stock transactions for UK Capital Gains Tax.\n\nAccepts either JSON array or DSL text format. Returns normalized JSON.\n\n## Transaction Format\n\nEach transaction is a JSON object with these fields:\n\n### Required fields:\n- date: \"YYYY-MM-DD\" (ISO 8601 format)\n- ticker: Stock symbol (case-insensitive, e.g., \"AAPL\", \"VOD\")\n- action: One of \"BUY\", \"SELL\", \"DIVIDEND\", \"CAPRETURN\", \"SPLIT\", \"UNSPLIT\" (case-insensitive)\n\n### Action-specific fields:\n\n**BUY/SELL:**\n- amount: Quantity of shares (positive number as string, e.g., \"100\")\n- price: Price per share - either:\n  - Plain string for GBP: \"150.00\"\n  - Object for foreign currency: {\"amount\": \"150.00\", \"currency\": \"USD\"}\n- fees: (optional) Transaction fees, same format as price\n\n**DIVIDEND:**\n- amount: Number of shares receiving dividend\n- total_value: Total dividend value (same format as price)\n- tax_paid: (optional) Tax withheld\n\n**CAPRETURN:**\n- amount: Number of shares\n- total_value: Total capital return value\n- fees: (optional) Fees\n\n**SPLIT/UNSPLIT:**\n- ratio: Split ratio as string (e.g., \"2\" for 2-for-1 split)\n\n### Currency:\n- Default is GBP. US stocks (AAPL, MSFT, etc.) trade in USD - specify currency!\n- Supported: GBP, USD, EUR, JPY, CHF, AUD, CAD, CNY, and other ISO 4217 codes\n\n### Examples:\n\nUS stock: [{\"date\":\"2024-01-15\",\"ticker\":\"AAPL\",\"action\":\"BUY\",\"amount\":\"100\",\"price\":{\"amount\":\"185.50\",\"currency\":\"USD\"}}]\n\nUK stock: [{\"date\":\"2024-01-15\",\"ticker\":\"VOD\",\"action\":\"BUY\",\"amount\":\"100\",\"price\":\"120.50\"}]"
    )]
    async fn parse_transactions(
        &self,
        Parameters(req): Parameters<ParseTransactionsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let transactions = self.parse_input(&req.transactions)?;
        let json = serde_json::to_string_pretty(&transactions).map_err(|e| {
            McpError::internal_error(format!("JSON serialization error: {e}"), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Calculate UK Capital Gains Tax report.\n\nApplies HMRC share matching rules (Same Day, Bed & Breakfast, Section 104) and returns gains/losses in GBP.\n\n### Parameters:\n- transactions: JSON array of transactions (see parse_transactions for schema)\n- year: (optional) Tax year START (e.g., 2024 for tax year 2024/25 running 6 April 2024 to 5 April 2025). If omitted, report includes all tax years with disposals.\n\n### Currency:\n- All results converted to GBP using HMRC monthly average rates\n- For US stocks, specify USD: \"price\": {\"amount\": \"150\", \"currency\": \"USD\"}\n- Without currency specified, amounts are treated as GBP\n\n### Example:\n\nInput:\n[{\"date\":\"2024-01-15\",\"ticker\":\"AAPL\",\"action\":\"BUY\",\"amount\":\"100\",\"price\":{\"amount\":\"150\",\"currency\":\"USD\"}},\n {\"date\":\"2024-06-20\",\"ticker\":\"AAPL\",\"action\":\"SELL\",\"amount\":\"50\",\"price\":{\"amount\":\"180\",\"currency\":\"USD\"}}]\n\nWith year=2024 returns tax report for 2024/25.\nWithout year, returns report for all years with disposals."
    )]
    async fn calculate_report(
        &self,
        Parameters(req): Parameters<CalculateReportRequest>,
    ) -> Result<CallToolResult, McpError> {
        let report = self.do_calculate_report(&req.transactions, req.year)?;
        let json = serde_json::to_string_pretty(&report).map_err(|e| {
            McpError::internal_error(format!("JSON serialization error: {e}"), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Explain how a specific stock sale was matched to acquisitions under HMRC rules.\n\n### HMRC Matching Rules (applied in order):\n1. **Same Day Rule**: Shares sold matched first to shares bought on the same day\n2. **Bed & Breakfast Rule**: Then matched to shares bought within 30 days AFTER the sale\n3. **Section 104 Pool**: Finally matched against the average cost pool of remaining shares\n\n### Parameters:\n- transactions: JSON array of all transactions (buys and sells)\n- disposal_date: Date of the sale to explain (YYYY-MM-DD)\n- ticker: Stock symbol (case-insensitive)\n\n### Response:\nShows how each portion of the sale was matched, the cost basis used, and resulting gain/loss (all in GBP).\n\n### Example:\nFor a sale of 100 shares where 30 were bought same day, 40 bought within 30 days after, and 30 from the pool:\n- Same Day: 30 shares matched at purchase cost\n- Bed & Breakfast: 40 shares matched at later purchase cost\n- Section 104: 30 shares matched at pool average cost"
    )]
    async fn explain_matching(
        &self,
        Parameters(req): Parameters<ExplainMatchingRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Parse the date to get the tax year
        let date =
            chrono::NaiveDate::parse_from_str(&req.disposal_date, "%Y-%m-%d").map_err(|_| {
                McpError::invalid_params(
                    format!(
                        "Invalid date format: {}. Expected YYYY-MM-DD.",
                        req.disposal_date
                    ),
                    None,
                )
            })?;

        // Calculate tax year from date (UK tax year starts April 6)
        let year = if date.month() < 4 || (date.month() == 4 && date.day() < 6) {
            date.year() - 1
        } else {
            date.year()
        };

        let report = self.do_calculate_report(&req.transactions, Some(year))?;
        let disposal = self.find_disposal(&report, &req.disposal_date, &req.ticker)?;

        // Build explanation
        let matches: Vec<MatchDetail> = disposal
            .matches
            .iter()
            .map(|m| {
                let rule_name = match m.rule {
                    MatchRule::SameDay => "Same Day",
                    MatchRule::BedAndBreakfast => "Bed & Breakfast",
                    MatchRule::Section104 => "Section 104",
                };

                let explanation = match m.rule {
                    MatchRule::SameDay => {
                        format!(
                            "Matched {} shares against same-day acquisition. Cost basis: {}",
                            m.quantity, m.allowable_cost
                        )
                    }
                    MatchRule::BedAndBreakfast => {
                        let acq_date = m
                            .acquisition_date
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        format!(
                            "Matched {} shares against acquisition on {} (within 30 days after sale). Cost basis: {}",
                            m.quantity, acq_date, m.allowable_cost
                        )
                    }
                    MatchRule::Section104 => {
                        format!(
                            "Matched {} shares from Section 104 pool at average cost. Cost basis: {}",
                            m.quantity, m.allowable_cost
                        )
                    }
                };

                MatchDetail {
                    rule: rule_name.to_string(),
                    quantity: m.quantity.to_string(),
                    allowable_cost: m.allowable_cost.to_string(),
                    gain_or_loss: m.gain_or_loss.to_string(),
                    acquisition_date: m.acquisition_date.map(|d| d.to_string()),
                    explanation,
                }
            })
            .collect();

        let total_gain_or_loss: Decimal = disposal.matches.iter().map(|m| m.gain_or_loss).sum();

        let explanation = MatchExplanation {
            disposal_date: disposal.date.to_string(),
            ticker: disposal.ticker.clone(),
            quantity: disposal.quantity.to_string(),
            proceeds: disposal.proceeds.to_string(),
            matches,
            total_gain_or_loss: total_gain_or_loss.to_string(),
        };

        let json = serde_json::to_string_pretty(&explanation).map_err(|e| {
            McpError::internal_error(format!("JSON serialization error: {e}"), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Get HMRC monthly average FX rate for converting foreign currency to GBP. Use these rates for non-GBP stock transactions.\n\nParameters:\n- currency: ISO 4217 code (e.g., \"USD\", \"EUR\")\n- year: The year (e.g., 2024)\n- month: The month (1-12)\n\nReturns the rate as units of foreign currency per 1 GBP."
    )]
    async fn get_fx_rate(
        &self,
        Parameters(req): Parameters<GetFxRateRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate month
        if req.month < 1 || req.month > 12 {
            return Err(McpError::invalid_params(
                format!(
                    "Invalid month: {}. Month must be between 1 and 12.\n\nExample: month=6 for June",
                    req.month
                ),
                None,
            ));
        }

        let cache = self
            .fx_cache
            .as_ref()
            .ok_or_else(|| McpError::internal_error("FX rates not available", None))?;

        // Normalize currency code to uppercase
        let currency_upper = req.currency.to_uppercase();

        let entry = cache
            .get(&currency_upper, req.year, req.month)
            .ok_or_else(|| {
                // Check if currency exists in any period
                let has_currency = (2015..=2025)
                    .any(|y| (1..=12).any(|m| cache.get(&currency_upper, y, m).is_some()));

                let hint = if has_currency {
                    HINT_FX_RATE_EXISTS
                } else {
                    HINT_FX_RATE_UNKNOWN
                };

                let msg = format!(
                    "No exchange rate found for {currency_upper} in {}-{:02}\n\n{hint}",
                    req.year, req.month
                );

                McpError::invalid_params(msg, None)
            })?;

        let response = FxRateResponse {
            rate: entry.rate_per_gbp.to_string(),
            currency: req.currency.to_uppercase(),
            period: format!("{}-{:02}", req.year, req.month),
        };

        let json = serde_json::to_string_pretty(&response).map_err(|e| {
            McpError::internal_error(format!("JSON serialization error: {e}"), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Convert JSON transactions to CGT DSL format for use with the cgt-tool CLI.\n\nThe DSL format is a text-based format that can be saved as .cgt files and processed by the cgt-tool command-line interface.\n\n### DSL Syntax:\n- BUY:      YYYY-MM-DD BUY TICKER QUANTITY @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]\n- SELL:     YYYY-MM-DD SELL TICKER QUANTITY @ PRICE [CURRENCY] [FEES AMOUNT [CURRENCY]]\n- DIVIDEND: YYYY-MM-DD DIVIDEND TICKER QUANTITY TOTAL VALUE [CURRENCY] [TAX AMOUNT [CURRENCY]]\n- SPLIT:    YYYY-MM-DD SPLIT TICKER RATIO VALUE\n- UNSPLIT:  YYYY-MM-DD UNSPLIT TICKER RATIO VALUE\n\nNote: FEES and TAX clauses are optional (default to 0 when omitted).\n\n### Example output:\n2024-01-15 BUY AAPL 100 @ 150 USD FEES 10 USD\n2024-06-20 SELL AAPL 50 @ 180 USD FEES 5 USD"
    )]
    async fn convert_to_dsl(
        &self,
        Parameters(req): Parameters<ConvertToDslRequest>,
    ) -> Result<CallToolResult, McpError> {
        let transactions = self.parse_input(&req.transactions)?;
        let dsl = self.transactions_to_dsl(&transactions)?;
        Ok(CallToolResult::success(vec![Content::text(dsl)]))
    }
}

impl CgtServer {
    /// Convert transactions to DSL format.
    fn transactions_to_dsl(&self, transactions: &[Transaction]) -> Result<String, McpError> {
        let mut lines = Vec::new();

        for tx in transactions {
            let line = self.transaction_to_dsl(tx)?;
            lines.push(line);
        }

        Ok(lines.join("\n"))
    }

    /// Convert a single transaction to DSL format.
    fn transaction_to_dsl(&self, tx: &Transaction) -> Result<String, McpError> {
        use cgt_core::Operation;

        let date = tx.date.format("%Y-%m-%d");

        match &tx.operation {
            Operation::Buy {
                amount,
                price,
                fees,
            } => {
                let mut line = format!(
                    "{} BUY {} {} @ {}",
                    date,
                    tx.ticker,
                    amount,
                    Self::format_currency_amount(price)
                );
                if !fees.amount.is_zero() {
                    line.push_str(&format!(" FEES {}", Self::format_currency_amount(fees)));
                }
                Ok(line)
            }
            Operation::Sell {
                amount,
                price,
                fees,
            } => {
                let mut line = format!(
                    "{} SELL {} {} @ {}",
                    date,
                    tx.ticker,
                    amount,
                    Self::format_currency_amount(price)
                );
                if !fees.amount.is_zero() {
                    line.push_str(&format!(" FEES {}", Self::format_currency_amount(fees)));
                }
                Ok(line)
            }
            Operation::Dividend {
                amount,
                total_value,
                tax_paid,
            } => {
                let mut line = format!(
                    "{} DIVIDEND {} {} TOTAL {}",
                    date,
                    tx.ticker,
                    amount,
                    Self::format_currency_amount(total_value)
                );
                if !tax_paid.amount.is_zero() {
                    line.push_str(&format!(" TAX {}", Self::format_currency_amount(tax_paid)));
                }
                Ok(line)
            }
            Operation::CapReturn {
                amount,
                total_value,
                fees,
            } => {
                let mut line = format!(
                    "{} CAPRETURN {} {} TOTAL {}",
                    date,
                    tx.ticker,
                    amount,
                    Self::format_currency_amount(total_value)
                );
                if !fees.amount.is_zero() {
                    line.push_str(&format!(" FEES {}", Self::format_currency_amount(fees)));
                }
                Ok(line)
            }
            Operation::Split { ratio } => {
                Ok(format!("{} SPLIT {} RATIO {}", date, tx.ticker, ratio))
            }
            Operation::Unsplit { ratio } => {
                Ok(format!("{} UNSPLIT {} RATIO {}", date, tx.ticker, ratio))
            }
        }
    }

    /// Format a CurrencyAmount for DSL output.
    fn format_currency_amount(amount: &CurrencyAmount) -> String {
        if amount.is_gbp() {
            // GBP can be written without currency code
            format!("{} GBP", amount.amount)
        } else {
            format!("{} {}", amount.amount, amount.code())
        }
    }
}

#[tool_handler]
impl ServerHandler for CgtServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(SERVER_INSTRUCTIONS.to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources = RESOURCES
            .iter()
            .map(|r| RawResource::new(r.uri, r.name).no_annotation())
            .collect();
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let resource = RESOURCES
            .iter()
            .find(|r| r.uri == request.uri)
            .ok_or_else(|| {
                McpError::resource_not_found(format!("Resource not found: {}", request.uri), None)
            })?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(resource.content, request.uri)],
        })
    }
}

/// Run the MCP server over stdio.
///
/// This function blocks until the server is shut down (stdin closes or signal received).
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a fatal error.
pub async fn run_server() -> Result<(), McpServerError> {
    let server = CgtServer::new()?;
    let transport = rmcp::transport::stdio();

    let service = server
        .serve(transport)
        .await
        .map_err(|e| McpServerError::Service(e.to_string()))?;

    // Block until the service completes or a shutdown signal is received.
    tokio::select! {
        result = service.waiting() => {
            // Service ended (stdin closed, error, etc.)
            if let Err(e) = result {
                let err_str = e.to_string();
                // These are expected shutdown conditions, not errors
                if err_str.contains("closed")
                    || err_str.contains("EOF")
                    || err_str.contains("broken pipe")
                    || err_str.contains("connection reset")
                {
                    return Ok(());
                }
                return Err(McpServerError::Service(err_str));
            }
        }
        _ = shutdown_signal() => {
            // Received shutdown signal (SIGTERM, SIGINT, etc.)
            // Just exit cleanly
        }
    }

    Ok(())
}

/// Block until a shutdown signal is received (SIGTERM, SIGINT on Unix; Ctrl+C on Windows).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let sigterm = signal(SignalKind::terminate()).ok();
        let sigint = signal(SignalKind::interrupt()).ok();
        let sighup = signal(SignalKind::hangup()).ok();

        tokio::select! {
            _ = async {
                if let Some(mut s) = sigterm { s.recv().await } else { std::future::pending().await }
            } => {}
            _ = async {
                if let Some(mut s) = sigint { s.recv().await } else { std::future::pending().await }
            } => {}
            _ = async {
                if let Some(mut s) = sighup { s.recv().await } else { std::future::pending().await }
            } => {}
        }
    }

    #[cfg(not(unix))]
    {
        // On Windows, just handle Ctrl+C
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_CGT: &str = r#"
2024-01-15 BUY AAPL 100 @ 150 GBP FEES 10 GBP
2024-06-20 SELL AAPL 50 @ 180 GBP FEES 5 GBP
"#;

    const BED_AND_BREAKFAST_CGT: &str = r#"
2024-01-15 BUY AAPL 100 @ 150 GBP
2024-06-20 SELL AAPL 50 @ 180 GBP
2024-06-25 BUY AAPL 30 @ 170 GBP
"#;

    /// Create a server without FX cache for testing GBP-only transactions.
    fn test_server_without_fx() -> CgtServer {
        CgtServer {
            fx_cache: None,
            tool_router: CgtServer::tool_router(),
        }
    }

    /// Create a server with FX cache for testing foreign currency transactions.
    fn test_server_with_fx() -> CgtServer {
        CgtServer::new().ok().unwrap_or_else(test_server_without_fx)
    }

    /// Extract text from CallToolResult content.
    fn extract_text(result: &CallToolResult) -> Option<&str> {
        result.content.first().and_then(|content| {
            if let RawContent::Text(text) = &content.raw {
                Some(text.text.as_str())
            } else {
                None
            }
        })
    }

    #[tokio::test]
    async fn test_parse_transactions_success() {
        let server = test_server_without_fx();
        let result = server
            .parse_transactions(Parameters(ParseTransactionsRequest {
                transactions: SIMPLE_CGT.to_string(),
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();
        assert!(!call_result.is_error.unwrap_or(false));

        // Verify JSON contains expected data
        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("AAPL"));
        assert!(text.contains("BUY"));
        assert!(text.contains("SELL"));
    }

    #[tokio::test]
    async fn test_parse_transactions_invalid_syntax() {
        let server = test_server_without_fx();
        let result = server
            .parse_transactions(Parameters(ParseTransactionsRequest {
                transactions: "this is not valid CGT DSL".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("Parsing error"));
    }

    #[tokio::test]
    async fn test_parse_transactions_empty_content() {
        let server = test_server_without_fx();
        let result = server
            .parse_transactions(Parameters(ParseTransactionsRequest {
                transactions: "".to_string(),
            }))
            .await;

        // Empty content should parse to empty array
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_calculate_report_success() {
        let server = test_server_without_fx();
        let result = server
            .calculate_report(Parameters(CalculateReportRequest {
                transactions: SIMPLE_CGT.to_string(),
                year: Some(2024),
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();
        assert!(!call_result.is_error.unwrap_or(false));

        // Verify JSON contains expected data
        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("tax_years"));
        assert!(text.contains("disposals"));
    }

    #[tokio::test]
    async fn test_calculate_report_invalid_content() {
        let server = test_server_without_fx();
        let result = server
            .calculate_report(Parameters(CalculateReportRequest {
                transactions: "invalid content".to_string(),
                year: Some(2024),
            }))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculate_report_no_disposals_in_year() {
        let server = test_server_without_fx();
        // Request tax year 2023 but transactions are in 2024
        let result = server
            .calculate_report(Parameters(CalculateReportRequest {
                transactions: SIMPLE_CGT.to_string(),
                year: Some(2023),
            }))
            .await;

        // Should succeed but with empty disposals for that year
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_calculate_report_all_years() {
        let server = test_server_without_fx();
        // Multi-year transactions spanning 2023/24 and 2024/25
        let multi_year_cgt = r#"
2023-06-01 BUY AAPL 100 @ 150 GBP
2023-12-15 SELL AAPL 50 @ 160 GBP
2024-06-20 SELL AAPL 30 @ 170 GBP
"#;
        let result = server
            .calculate_report(Parameters(CalculateReportRequest {
                transactions: multi_year_cgt.to_string(),
                year: None,
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();
        assert!(!call_result.is_error.unwrap_or(false));

        // Verify JSON contains multiple tax years
        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("tax_years"));
        // Should have both 2023/24 and 2024/25
        assert!(text.contains("2023/24"));
        assert!(text.contains("2024/25"));
    }

    #[tokio::test]
    async fn test_explain_matching_section104() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: SIMPLE_CGT.to_string(),
                disposal_date: "2024-06-20".to_string(),
                ticker: "AAPL".to_string(),
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();

        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("Section 104"));
        assert!(text.contains("AAPL"));
        assert!(text.contains("2024-06-20"));
    }

    #[tokio::test]
    async fn test_explain_matching_bed_and_breakfast() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: BED_AND_BREAKFAST_CGT.to_string(),
                disposal_date: "2024-06-20".to_string(),
                ticker: "AAPL".to_string(),
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();

        let text = extract_text(&call_result).expect("Expected text content");
        // Should show B&B match for 30 shares and S104 for remaining 20
        assert!(text.contains("Bed & Breakfast"));
    }

    #[tokio::test]
    async fn test_explain_matching_invalid_date_format() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: SIMPLE_CGT.to_string(),
                disposal_date: "20/06/2024".to_string(), // Wrong format
                ticker: "AAPL".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("Invalid date format"));
    }

    #[tokio::test]
    async fn test_explain_matching_disposal_not_found() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: SIMPLE_CGT.to_string(),
                disposal_date: "2024-06-21".to_string(), // Wrong date
                ticker: "AAPL".to_string(),
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("No disposal found"));
        // Should suggest available dates
        assert!(err.message.contains("2024-06-20"));
    }

    #[tokio::test]
    async fn test_explain_matching_ticker_not_found() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: SIMPLE_CGT.to_string(),
                disposal_date: "2024-06-20".to_string(),
                ticker: "GOOG".to_string(), // Wrong ticker
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.message
                .contains("No SELL transactions found for ticker"),
            "Expected helpful error message, got: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_explain_matching_case_insensitive_ticker() {
        let server = test_server_without_fx();
        let result = server
            .explain_matching(Parameters(ExplainMatchingRequest {
                transactions: SIMPLE_CGT.to_string(),
                disposal_date: "2024-06-20".to_string(),
                ticker: "aapl".to_string(), // Lowercase
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_fx_rate_success() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            // Skip test if FX cache couldn't be loaded
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 2024,
                month: 1,
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();

        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("USD"));
        assert!(text.contains("2024-01"));
        assert!(text.contains("rate"));
    }

    #[tokio::test]
    async fn test_get_fx_rate_case_insensitive() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "usd".to_string(), // Lowercase
                year: 2024,
                month: 1,
            }))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_fx_rate_not_found() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 1900, // Year with no rates
                month: 1,
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("No exchange rate found"));
    }

    #[tokio::test]
    async fn test_get_fx_rate_no_cache() {
        let server = test_server_without_fx();
        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 2024,
                month: 1,
            }))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("FX rates not available"));
    }

    #[test]
    fn test_parse_input_success() {
        let server = test_server_without_fx();
        let result = server.parse_input(SIMPLE_CGT);
        assert!(result.is_ok());
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_parse_input_invalid() {
        let server = test_server_without_fx();
        let result = server.parse_input("invalid content");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_json() {
        let server = test_server_without_fx();
        // JSON format: plain strings for GBP amounts
        let json_input = r#"[
            {
                "date": "2024-01-15",
                "ticker": "AAPL",
                "action": "BUY",
                "amount": "100",
                "price": "150",
                "fees": "10"
            },
            {
                "date": "2024-06-20",
                "ticker": "AAPL",
                "action": "SELL",
                "amount": "50",
                "price": "180",
                "fees": "5"
            }
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].ticker, "AAPL");
        assert!(matches!(
            transactions[0].operation,
            cgt_core::Operation::Buy { .. }
        ));
        assert!(matches!(
            transactions[1].operation,
            cgt_core::Operation::Sell { .. }
        ));
    }

    #[test]
    fn test_parse_input_json_invalid() {
        let server = test_server_without_fx();
        // Invalid JSON array
        let result = server.parse_input("[{invalid json}]");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("JSON parse error"));
    }

    #[test]
    fn test_parse_input_json_gbp_as_object() {
        let server = test_server_without_fx();
        // GBP can also be specified as an object with currency field
        let json_input = r#"[{
            "date": "2024-01-15",
            "ticker": "AAPL",
            "action": "BUY",
            "amount": "100",
            "price": {"amount": "150", "currency": "GBP"},
            "fees": {"amount": "10", "currency": "GBP"}
        }]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "GBP as object should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_input_json_foreign_currency_parses_successfully() {
        let server = test_server_without_fx();
        // Foreign currency should parse successfully (conversion happens at calculation time)
        let json_input = r#"[{
            "date": "2024-01-15",
            "ticker": "AAPL",
            "action": "BUY",
            "amount": "100",
            "price": {"amount": "150", "currency": "USD"},
            "fees": {"amount": "10", "currency": "USD"}
        }]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "Foreign currency should parse: {:?}",
            result.err()
        );
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 1);
    }

    #[test]
    fn test_parse_input_json_without_fees() {
        let server = test_server_without_fx();
        // JSON without fees field should default to zero (using plain string for GBP price)
        let json_input = r#"[
            {"date": "2023-12-21", "ticker": "VOD", "action": "BUY", "amount": "5", "price": "11"},
            {"date": "2023-12-31", "ticker": "VOD", "action": "SELL", "amount": "4", "price": "12"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "JSON without fees should parse: {:?}",
            result.err()
        );
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_calculate_report_json_without_fees() {
        let server = test_server_without_fx();
        // Calculate report with JSON missing fees (using plain string for GBP price)
        let json_input = r#"[
            {"date": "2023-12-21", "ticker": "VOD", "action": "BUY", "amount": "5", "price": "11"},
            {"date": "2023-12-31", "ticker": "VOD", "action": "SELL", "amount": "4", "price": "12"}
        ]"#;

        let result = server.do_calculate_report(json_input, Some(2023));
        assert!(
            result.is_ok(),
            "Calculate report should work: {:?}",
            result.err()
        );
        let report = result.ok().unwrap();
        assert!(!report.tax_years.is_empty());
    }

    #[test]
    fn test_parse_input_json_object_without_currency_errors() {
        let server = test_server_without_fx();
        // Object format without currency should error with helpful message
        let json_input = r#"[
            {"date": "2023-12-21", "ticker": "AAPL", "action": "BUY", "amount": "5", "price": {"amount": "11"}}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "Should reject object without currency");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("currency"),
            "Error should mention currency: {}",
            err.message
        );
        assert!(
            err.message.contains("plain string"),
            "Error should suggest plain string: {}",
            err.message
        );
    }

    #[test]
    fn test_parse_input_json_split() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": "150"},
            {"date": "2024-06-01", "ticker": "AAPL", "action": "SPLIT", "ratio": "2"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_ok(), "SPLIT should parse: {:?}", result.err());
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_parse_input_json_dividend() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "VWRL", "action": "BUY", "amount": "100", "price": "85"},
            {"date": "2024-03-01", "ticker": "VWRL", "action": "DIVIDEND", "amount": "100", "total_value": "50", "tax_paid": "0"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_ok(), "DIVIDEND should parse: {:?}", result.err());
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_ticker_case_insensitive() {
        let server = test_server_without_fx();
        // Mix of uppercase and lowercase tickers should all be treated as same ticker
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "aapl", "action": "BUY", "amount": "100", "price": "150"},
            {"date": "2024-06-20", "ticker": "AAPL", "action": "SELL", "amount": "50", "price": "180"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_ok(), "Should parse: {:?}", result.err());
        let transactions = result.ok().unwrap();

        // Both tickers should be normalized to uppercase
        assert_eq!(transactions[0].ticker, "AAPL");
        assert_eq!(transactions[1].ticker, "AAPL");
    }

    #[test]
    fn test_calculate_report_mixed_case_tickers() {
        let server = test_server_without_fx();
        // Calculate report with mixed case tickers - should match correctly
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "vod", "action": "BUY", "amount": "100", "price": "120"},
            {"date": "2024-06-20", "ticker": "VOD", "action": "SELL", "amount": "50", "price": "130"}
        ]"#;

        let result = server.do_calculate_report(json_input, Some(2024));
        assert!(result.is_ok(), "Should calculate: {:?}", result.err());
        let report = result.ok().unwrap();

        // Should have one disposal
        assert_eq!(report.tax_years.len(), 1);
        assert_eq!(report.tax_years[0].disposals.len(), 1);
        assert_eq!(report.tax_years[0].disposals[0].ticker, "VOD");
    }

    #[test]
    fn test_do_calculate_report_success() {
        let server = test_server_without_fx();
        let result = server.do_calculate_report(SIMPLE_CGT, Some(2024));
        assert!(result.is_ok());
        let report = result.ok().unwrap();
        assert!(!report.tax_years.is_empty());
    }

    #[test]
    fn test_find_disposal_success() {
        let server = test_server_without_fx();
        let report = server
            .do_calculate_report(SIMPLE_CGT, Some(2024))
            .ok()
            .unwrap();
        let result = server.find_disposal(&report, "2024-06-20", "AAPL");
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_disposal_not_found() {
        let server = test_server_without_fx();
        let report = server
            .do_calculate_report(SIMPLE_CGT, Some(2024))
            .ok()
            .unwrap();
        let result = server.find_disposal(&report, "2024-06-21", "AAPL");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_creation() {
        // This test verifies the server can be created with FX cache
        let result = CgtServer::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_info() {
        let server = test_server_without_fx();
        let info = server.get_info();

        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
        assert!(info.instructions.is_some());
        assert!(
            info.instructions
                .as_ref()
                .is_some_and(|i| i.contains("Capital Gains Tax"))
        );
    }

    #[test]
    fn test_action_case_insensitive() {
        let server = test_server_without_fx();
        // Actions should be case-insensitive
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "buy", "amount": "100", "price": "150"},
            {"date": "2024-06-20", "ticker": "AAPL", "action": "sell", "amount": "50", "price": "180"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "Lowercase actions should work: {:?}",
            result.err()
        );
        let transactions = result.ok().unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_action_mixed_case() {
        let server = test_server_without_fx();
        // Mixed case actions should work
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "Buy", "amount": "100", "price": "150"},
            {"date": "2024-06-20", "ticker": "AAPL", "action": "Sell", "amount": "50", "price": "180"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "Mixed case actions should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_invalid_action_error_message() {
        let server = test_server_without_fx();
        // Invalid action should show helpful error with valid options
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "HOLD", "amount": "100", "price": "150"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.message.contains("HOLD"),
            "Error should mention invalid action: {}",
            err.message
        );
        assert!(
            err.message.contains("BUY"),
            "Error should list valid actions: {}",
            err.message
        );
    }

    #[test]
    fn test_missing_required_field_error() {
        let server = test_server_without_fx();
        // Missing required field should show helpful error
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.message.contains("price"),
            "Error should mention missing field: {}",
            err.message
        );
    }

    #[test]
    fn test_split_lowercase() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": "150"},
            {"date": "2024-06-01", "ticker": "AAPL", "action": "split", "ratio": "2"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "Lowercase split should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_unsplit_parsing() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": "150"},
            {"date": "2024-06-01", "ticker": "AAPL", "action": "UNSPLIT", "ratio": "2"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_ok(), "UNSPLIT should parse: {:?}", result.err());
    }

    #[test]
    fn test_dividend_lowercase() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "VWRL", "action": "buy", "amount": "100", "price": "85"},
            {"date": "2024-03-01", "ticker": "VWRL", "action": "dividend", "amount": "100", "total_value": "50"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "Lowercase dividend should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_invalid_date_error_message() {
        let server = test_server_without_fx();
        // Invalid date (month 13)
        let json_input = r#"[
            {"date": "2024-13-01", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": "150"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        // Should contain the date string in the error
        assert!(
            err.message.contains("2024-13-01") || err.message.contains("date"),
            "Error should reference date: {}",
            err.message
        );
    }

    #[test]
    fn test_negative_amount_rejected() {
        let server = test_server_without_fx();
        // Negative amounts should be rejected with helpful error
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "-100", "price": "150"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.message.contains("positive") || err.message.contains("-100"),
            "Error should mention positive or the negative value: {}",
            err.message
        );
    }

    #[test]
    fn test_zero_amount_rejected() {
        let server = test_server_without_fx();
        // Zero amounts should also be rejected
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "0", "price": "150"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(
            err.message.contains("positive"),
            "Error should mention positive: {}",
            err.message
        );
    }

    #[test]
    fn test_convert_to_dsl_basic() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": "150"},
            {"date": "2024-06-20", "ticker": "AAPL", "action": "SELL", "amount": "50", "price": "180"}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        assert!(dsl.contains("2024-01-15 BUY AAPL 100 @"));
        assert!(dsl.contains("2024-06-20 SELL AAPL 50 @"));
        assert!(dsl.contains("GBP")); // Default currency
    }

    #[test]
    fn test_convert_to_dsl_with_fees() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "VOD", "action": "BUY", "amount": "100", "price": "120", "fees": "10"}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        assert!(dsl.contains("FEES 10 GBP"));
    }

    #[test]
    fn test_convert_to_dsl_foreign_currency() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "AAPL", "action": "BUY", "amount": "100", "price": {"amount": "150", "currency": "USD"}, "fees": {"amount": "10", "currency": "USD"}}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        assert!(dsl.contains("150 USD"));
        assert!(dsl.contains("FEES 10 USD"));
    }

    #[test]
    fn test_convert_to_dsl_split() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-06-01", "ticker": "NVDA", "action": "SPLIT", "ratio": "4"}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        assert!(dsl.contains("2024-06-01 SPLIT NVDA RATIO 4"));
    }

    #[test]
    fn test_convert_to_dsl_dividend() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-03-01", "ticker": "VWRL", "action": "DIVIDEND", "amount": "100", "total_value": "50", "tax_paid": "5"}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        assert!(dsl.contains("2024-03-01 DIVIDEND VWRL 100 TOTAL 50 GBP TAX 5 GBP"));
    }

    #[test]
    fn test_convert_to_dsl_dividend_zero_tax_omitted() {
        let server = test_server_without_fx();
        // tax_paid is 0 (or omitted), so TAX clause should not appear in output
        let json_input = r#"[
            {"date": "2024-03-01", "ticker": "VWRL", "action": "DIVIDEND", "amount": "100", "total_value": "50", "tax_paid": "0"}
        ]"#;

        let transactions = server.parse_input(json_input).expect("should parse");
        let dsl = server
            .transactions_to_dsl(&transactions)
            .expect("should convert");

        // TAX 0 should be omitted from output
        assert!(dsl.contains("2024-03-01 DIVIDEND VWRL 100 TOTAL 50 GBP"));
        assert!(
            !dsl.contains("TAX"),
            "TAX clause should be omitted when tax is 0: {dsl}"
        );
    }

    #[tokio::test]
    async fn test_convert_to_dsl_tool() {
        let server = test_server_without_fx();
        let result = server
            .convert_to_dsl(Parameters(ConvertToDslRequest {
                transactions: r#"[{"date":"2024-01-15","ticker":"AAPL","action":"BUY","amount":"100","price":"150"}]"#.to_string(),
            }))
            .await;

        assert!(result.is_ok());
        let call_result = result.ok().unwrap();
        let text = extract_text(&call_result).expect("Expected text content");
        assert!(text.contains("2024-01-15 BUY AAPL 100"));
    }

    // Tests for error cases from tester report

    #[test]
    fn test_split_requires_ratio_field() {
        let server = test_server_without_fx();
        // WRONG format: using amount+price instead of ratio
        let json_input = r#"[
            {"date": "2024-06-10", "ticker": "NVDA", "action": "SPLIT", "amount": "10", "price": "0"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "SPLIT with amount+price should fail");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("ratio") || err.message.contains("missing field"),
            "Error should mention 'ratio' field: {}",
            err.message
        );
    }

    #[test]
    fn test_split_correct_format() {
        let server = test_server_without_fx();
        // CORRECT format: using ratio field
        let json_input = r#"[
            {"date": "2024-06-10", "ticker": "NVDA", "action": "SPLIT", "ratio": "10"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "SPLIT with ratio should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_unsplit_requires_ratio_field() {
        let server = test_server_without_fx();
        // WRONG format: using amount+price instead of ratio
        let json_input = r#"[
            {"date": "2024-01-15", "ticker": "TEST", "action": "UNSPLIT", "amount": "10", "price": "0"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "UNSPLIT with amount+price should fail");
    }

    #[test]
    fn test_dividend_requires_total_value() {
        let server = test_server_without_fx();
        // WRONG format: using price instead of total_value
        let json_input = r#"[
            {"date": "2024-03-15", "ticker": "VOD", "action": "DIVIDEND", "amount": "100", "price": "0.05"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "DIVIDEND with price should fail");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("total_value") || err.message.contains("missing field"),
            "Error should mention 'total_value' field: {}",
            err.message
        );
    }

    #[test]
    fn test_dividend_correct_format() {
        let server = test_server_without_fx();
        // CORRECT format: using total_value
        let json_input = r#"[
            {"date": "2024-03-15", "ticker": "VOD", "action": "DIVIDEND", "amount": "100", "total_value": "5.00"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(
            result.is_ok(),
            "DIVIDEND with total_value should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_invalid_date_day_32() {
        let server = test_server_without_fx();
        let json_input = r#"[
            {"date": "2024-01-32", "ticker": "VOD", "action": "BUY", "amount": "100", "price": "100"}
        ]"#;

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "Day 32 should be invalid");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("2024-01-32") || err.message.contains("date"),
            "Error should reference the invalid date: {}",
            err.message
        );
    }

    #[test]
    fn test_invalid_json_not_array() {
        let server = test_server_without_fx();
        let json_input = "this is not valid json";

        let result = server.parse_input(json_input);
        assert!(result.is_err(), "Invalid text should fail");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("Parse") || err.message.contains("DSL"),
            "Error should mention parsing: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_fx_rate_invalid_month_zero() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 2024,
                month: 0,
            }))
            .await;

        assert!(result.is_err(), "Month 0 should be invalid");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("between 1 and 12"),
            "Error should mention valid month range: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_fx_rate_invalid_month_13() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 2024,
                month: 13,
            }))
            .await;

        assert!(result.is_err(), "Month 13 should be invalid");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("between 1 and 12"),
            "Error should mention valid month range: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_fx_rate_unknown_currency() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "XYZ".to_string(),
                year: 2024,
                month: 6,
            }))
            .await;

        assert!(result.is_err(), "Unknown currency should fail");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("XYZ") && err.message.contains("ISO 4217"),
            "Error should mention the currency and ISO codes: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_fx_rate_future_date() {
        let server = test_server_with_fx();
        if server.fx_cache.is_none() {
            return;
        }

        let result = server
            .get_fx_rate(Parameters(GetFxRateRequest {
                currency: "USD".to_string(),
                year: 2030,
                month: 12,
            }))
            .await;

        assert!(result.is_err(), "Future date should fail");
        let err = result.err().unwrap();
        assert!(
            err.message.contains("not available") || err.message.contains("2030"),
            "Error should mention rate not available: {}",
            err.message
        );
    }
}
