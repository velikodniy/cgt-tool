# Tasks: Add MCP Server

## 1. Setup

- [x] 1.1 Create `crates/cgt-mcp/` crate with Cargo.toml (library)
- [x] 1.2 Add `rmcp`, `tokio`, `cgt-core`, `cgt-money` dependencies to cgt-mcp
- [x] 1.3 Add cgt-mcp to workspace in root Cargo.toml
- [x] 1.4 Add cgt-mcp dependency to cgt-cli Cargo.toml
- [x] 1.5 Add `Mcp` variant to Commands enum in cgt-cli/commands.rs

## 2. Core MCP Infrastructure (in cgt-mcp)

- [x] 2.1 Implement MCP server struct with stdio transport
- [x] 2.2 Implement tool registration and dispatch
- [x] 2.3 Add error handling mapping cgt-core errors to MCP error responses
- [x] 2.4 Export `run_server()` async entry point
- [x] 2.5 Add graceful shutdown on SIGTERM/SIGINT/SIGHUP and stdin close

## 3. CLI Integration (in cgt-cli)

- [x] 3.1 Wire up `mcp` subcommand in main.rs to call `cgt_mcp::run_server()`

## 4. Tools Implementation (in cgt-mcp)

- [x] 4.1 Implement `parse_transactions` tool
- [x] 4.2 Implement `calculate_report` tool
- [x] 4.3 Implement `explain_matching` tool
- [x] 4.4 Implement `get_fx_rate` tool

## 5. Resources Implementation (in cgt-mcp)

- [x] 5.1 Embed TAX_RULES.md at compile time
- [x] 5.2 Implement `tax-rules` resource endpoint
- [x] 5.3 Create DSL syntax documentation
- [x] 5.4 Implement `dsl-syntax` resource endpoint

## 6. Testing

- [x] 6.1 Add unit tests for each tool in cgt-mcp (36 tests)
- [x] 6.2 Add integration tests with mock MCP client (tested via unit tests)
- [x] 6.3 Test error handling (invalid input, missing rates, etc.)

## 7. Documentation

- [x] 7.1 Update README with `cgt-tool mcp` usage
- [x] 7.2 Add Claude Desktop configuration example to README
- [x] 7.3 Update tool descriptions with clear currency format requirements

## 8. Validation

- [x] 8.1 Run `cargo clippy` on workspace
- [x] 8.2 Run `cargo test` for all workspace crates (240 tests pass)
- [x] 8.3 Manual test with Claude Desktop

## 9. Improvements (post-initial implementation)

- [x] 9.1 Make `fees` field optional (defaults to 0)
- [x] 9.2 Require explicit currency when using object format for amounts
- [x] 9.3 Add detailed error messages with hints and examples
- [x] 9.4 Update SERVER_INSTRUCTIONS to emphasize USD for US stocks
- [x] 9.5 Add signal handling for graceful shutdown
