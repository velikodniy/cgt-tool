## 1. Implementation

- [ ] 1.1 Document Schwab JSON transactions and awards structures in new `docs/` file; relocate `TAX_RULES.md` into `docs/` with updated links and clarify wording if needed.
- [ ] 1.2 Update Schwab converter parsing to accept JSON transactions and JSON awards only, removing CSV handling and updating errors accordingly.
- [ ] 1.3 Adjust RSU mapping logic for Schwab JSON (vest date, FMV, quantities, withholding), update fixtures to JSON.
- [ ] 1.4 Update CLI help text, README, AGENTS, MCP tool/resource docs, and other references to reflect JSON-only Schwab conversion.
- [ ] 1.5 Run relevant tests (`cargo test -p cgt-converter`), `cargo fmt`, and `cargo clippy`.
