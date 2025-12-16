## 1. Rename Binary

- [x] 1.1 Add `[[bin]]` section to `crates/cgt-cli/Cargo.toml` with `name = "cgt-tool"`

## 2. Update Release Workflow

- [x] 2.1 Update `.github/workflows/release.yml` artifact names from `cgt-cli-*` to `cgt-tool-*`

## 3. Update Documentation

- [x] 3.1 Update `README.md` binary references and examples
- [x] 3.2 Update `AGENTS.md` command examples (no changes needed - no cgt-cli references)
- [x] 3.3 Update `openspec/project.md` crate description
- [x] 3.4 Update `openspec/specs/cli/spec.md` command examples

## 4. Validation

- [x] 4.1 Run `cargo build -p cgt-cli` and verify binary is named `cgt-tool`
- [x] 4.2 Run `cargo test` to ensure all tests pass
- [x] 4.3 Verify `./target/debug/cgt-tool --help` works

## Additional Changes Made

- Updated `crates/cgt-cli/tests/cli_tests.rs` to reference `cgt-tool` binary name
