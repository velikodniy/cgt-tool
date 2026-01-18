## MODIFIED Requirements

### Requirement: Tax Rules Resource

The system SHALL provide `tax-rules` resource with HMRC share matching documentation.

#### Scenario: Read tax rules

- **WHEN** client requests `cgt://docs/tax-rules` resource
- **THEN** return `docs/tax-rules.md` content as text/markdown
