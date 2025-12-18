## ADDED Requirements

### Requirement: MCP Subcommand

The system SHALL provide `mcp` subcommand to start an MCP server over stdio.

#### Scenario: Start MCP server

- **WHEN** `cgt-tool mcp` is run
- **THEN** start MCP server listening on stdin
- **AND** write JSON-RPC responses to stdout
- **AND** block until stdin is closed or server is terminated

#### Scenario: MCP help

- **WHEN** `cgt-tool mcp --help` is run
- **THEN** display MCP subcommand help with description of available tools
