# Change: Add MCP Server for AI Assistant Integration

## Why

AI assistants (Claude, GPT, etc.) can provide powerful natural language interfaces to complex tools, but they need a standardized way to invoke functionality. MCP (Model Context Protocol) is Anthropic's open protocol for this purpose. Adding an MCP server would let users ask natural language questions like "What's my capital gain for 2024?" or "Explain how my AAPL sale was matched" and get accurate answers powered by the existing CGT calculation engine.

Reference: https://github.com/velikodniy/cgt-tool/issues/2

## What Changes

- **NEW** `cgt-tool mcp` subcommand: Starts MCP server over stdio
- **NEW** MCP tools:
  - `parse_transactions` - Parse .cgt content and return structured transaction data
  - `calculate_report` - Generate CGT report for a specific tax year
  - `explain_matching` - Explain how a disposal was matched (Same Day/B&B/S104)
  - `get_fx_rate` - Get HMRC exchange rate for a currency/month
- **NEW** MCP resources:
  - `tax-rules` - Expose TAX_RULES.md as reference documentation
  - `dsl-syntax` - DSL grammar reference for transaction format

## Impact

- Affected specs: `cli` (modified), new spec `mcp-server`
- Affected code:
  - `crates/cgt-mcp/` - New library crate with MCP server logic
  - `crates/cgt-cli/` - New `mcp` subcommand dispatching to cgt-mcp
- Dependencies: `rmcp` and `tokio` in cgt-mcp; cgt-cli depends on cgt-mcp
- CI: No changes needed (same binary)
