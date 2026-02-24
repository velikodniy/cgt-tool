## ADDED Requirements

### Requirement: Unified validation issue type

The validation module SHALL represent all validation issues (errors and warnings) using a single `ValidationIssue` struct with a `Severity` enum distinguishing between `Error` and `Warning` severity levels.

#### Scenario: Error display format preserved

- **WHEN** a `ValidationIssue` with `Severity::Error` and a line number is displayed
- **THEN** the output SHALL match the format `Error (line N): TICKER on DATE - MESSAGE`

#### Scenario: Warning display format preserved

- **WHEN** a `ValidationIssue` with `Severity::Warning` and a line number is displayed
- **THEN** the output SHALL match the format `Warning (line N): TICKER on DATE - MESSAGE`

#### Scenario: Display without line number

- **WHEN** a `ValidationIssue` with `line: None` is displayed
- **THEN** the output SHALL omit the `(line N)` portion

#### Scenario: ValidationResult retains separate access

- **WHEN** validation produces both errors and warnings
- **THEN** `ValidationResult` SHALL provide separate `errors` and `warnings` fields, both typed as `Vec<ValidationIssue>`
