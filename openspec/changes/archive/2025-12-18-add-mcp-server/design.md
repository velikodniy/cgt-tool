# Design: MCP Server

## Context

MCP (Model Context Protocol) is a JSON-RPC 2.0 based protocol that allows AI assistants to discover and invoke external tools. The server communicates over stdio, making it easy to integrate with Claude Desktop, VS Code extensions, and other MCP-compatible clients.

The CGT tool already has well-factored, IO-free core libraries that can be directly exposed through MCP without modification.

## Goals / Non-Goals

**Goals:**

- Expose existing CGT functionality to AI assistants via MCP
- Maintain IO-free core principle (MCP server handles IO, core stays pure)
- Simple stdio-based server matching MCP specification
- Single binary distribution (`cgt-tool mcp` subcommand)

**Non-Goals:**

- Stateful session management (each tool call is independent)
- Interactive prompts or wizards
- Real-time streaming of partial results
- GUI or web interface
- Broker conversion via MCP (out of scope for initial implementation)

## Decisions

### Decision: Use `rmcp` crate for MCP implementation

The Rust MCP SDK (`rmcp`) provides a typed, async implementation of the MCP protocol with derive macros for tool definitions. This matches the project's Rust-first approach.

**Alternatives considered:**

- Custom JSON-RPC implementation: More control but significant boilerplate
- TypeScript SDK: Official but would require a separate Node.js process

### Decision: Stdio transport only

MCP supports multiple transports (stdio, SSE, WebSocket). Stdio is simplest and sufficient for local AI assistant integration.

**Alternatives considered:**

- SSE/WebSocket: Useful for web clients but adds complexity; can add later if needed

### Decision: Stateless tool calls

Each MCP tool call receives all required input and returns complete output. No state is maintained between calls. This matches the existing CLI design and keeps the server simple.

**Alternatives considered:**

- Maintain parsed transactions in memory: Would reduce redundant parsing but adds complexity and session management

### Decision: File content passed as parameters

Tools accept file content as string parameters rather than file paths. This:

- Matches the IO-free principle of existing crates
- Works with MCP clients that may not have filesystem access
- Allows the AI to pass content from various sources

### Decision: Subcommand of cgt-tool with separate library crate

The MCP server is exposed as `cgt-tool mcp` subcommand, but the implementation lives in a separate `cgt-mcp` library crate. This:

- Maintains single binary distribution
- Keeps CLI crate focused on dispatch only
- Allows MCP logic to be tested independently
- Follows existing crate separation pattern (cgt-core, cgt-formatter-\*, etc.)

## Architecture

```
cgt-cli (binary)
    └── mcp subcommand → calls cgt_mcp::run_server()

cgt-mcp (library)
    ├── Uses rmcp for MCP protocol
    ├── Imports cgt-core for parsing/calculation
    └── Imports cgt-money for FX rates
    └── Exports run_server() entry point

Tool flow:
1. AI sends JSON-RPC request via stdio
2. cgt-mcp deserializes, validates parameters
3. Calls appropriate cgt-core function
4. Returns JSON-RPC response with result or error
```

## Tool Definitions

### `parse_transactions`

- **Input:** `cgt_content: string` (DSL text)
- **Output:** JSON array of Transaction objects
- **Errors:** Parse errors with line numbers

### `calculate_report`

- **Input:** `cgt_content: string`, `year: integer`
- **Output:** JSON CGT report (same schema as `cgt-tool report --format json`)
- **Errors:** Parse/calculation errors

### `explain_matching`

- **Input:** `cgt_content: string`, `disposal_date: string`, `ticker: string`
- **Output:** Structured explanation of matching rule applied, matched acquisitions, and calculation
- **Errors:** If disposal not found or parse error

### `get_fx_rate`

- **Input:** `currency: string`, `year: integer`, `month: integer`
- **Output:** `{ rate: string, currency: string, month: string }`
- **Errors:** If rate not found

## Resource Definitions

### `tax-rules`

- **URI:** `cgt://docs/tax-rules`
- **Description:** HMRC share matching rules reference
- **Content:** TAX_RULES.md embedded at compile time

### `dsl-syntax`

- **URI:** `cgt://docs/dsl-syntax`
- **Description:** CGT DSL transaction format reference
- **Content:** Generated from parser.pest or dedicated documentation

## Risks / Trade-offs

| Risk                                  | Mitigation                                                                       |
| ------------------------------------- | -------------------------------------------------------------------------------- |
| `rmcp` crate is relatively new        | Pin version, test thoroughly; protocol is simple enough to reimplement if needed |
| Large file content in tool parameters | Document size limits; MCP clients typically handle this                          |
| No async IO in cgt-core               | Not needed; calculations are fast and don't block                                |
| Adding tokio/rmcp deps to CLI         | Acceptable trade-off for single binary; dependencies are well-maintained         |

## Future Extensions

- `convert_broker` tool for Schwab/other broker conversions (deferred)
- Additional resources (example files, FAQ)
- SSE transport for web clients
