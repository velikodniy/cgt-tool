# Release Procedure

## 1. Create semantic commits

Group changes logically:

- `fix:` for bug fixes
- `feat:` for new features
- `test:` for test additions/updates
- `docs:` for documentation
- `chore:` for version bumps, CI changes

## 2. Bump version

Update `version` in `[workspace.package]` in the root `Cargo.toml` (all crates inherit via `version.workspace = true`):

```bash
# Only the root workspace version needs updating
sed -i '' 's/^version = "X.Y.Z"/version = "X.Y.W"/' Cargo.toml
cargo check  # Verify Cargo.lock updates
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.W"
```

## 3. Push changes

```bash
git push
```

## 4. Create annotated tag

The CI extracts release notes from the tag message:

```bash
git tag -a vX.Y.W -m "vX.Y.W - Brief Description

## What's Changed

### Bug Fixes
- Description of fix

### Features
- Description of feature

**Full Changelog**: https://github.com/OWNER/REPO/compare/vPREV...vX.Y.W"
```

- First line becomes the release title
- Remaining lines become the release body

## 5. Push the tag

This triggers the release workflow:

```bash
git push origin vX.Y.W
```

## 6. Verify release

Check the release page and ensure CI passes.

## Pre-Release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Clippy clean (`cargo clippy`)
- [ ] Review `docs/spec.md` for accuracy against current behavior
- [ ] Cross-validation passes (`python3 scripts/cross-validate.py tests/inputs/*.cgt`)
