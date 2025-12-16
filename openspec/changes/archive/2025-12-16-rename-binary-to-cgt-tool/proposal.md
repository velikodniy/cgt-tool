# Change: Rename binary from cgt-cli to cgt-tool

## Why

The binary name `cgt-cli` is inconsistent with the project name `cgt-tool`. Renaming to `cgt-tool` provides better brand consistency and is more intuitive for users discovering the tool.

## What Changes

- Rename the CLI binary from `cgt-cli` to `cgt-tool`
- Update release workflow to produce `cgt-tool-*` artifacts
- Update all documentation references

## Impact

- Affected specs: cli
- Affected code: `crates/cgt-cli/Cargo.toml`, `.github/workflows/release.yml`
- Affected docs: `README.md`, `AGENTS.md`, `openspec/project.md`, `openspec/specs/cli/spec.md`
- **BREAKING**: Users with existing scripts referencing `cgt-cli` will need to update them
