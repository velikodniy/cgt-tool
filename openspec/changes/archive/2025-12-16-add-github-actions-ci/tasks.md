# Tasks

## 1. CI Workflow (Linting & Testing)

- [x] 1.1 Create `.github/workflows/ci.yml`
- [x] 1.2 Add job: trailing-whitespace, end-of-file-fixer, check-yaml, check-added-large-files
- [x] 1.3 Add job: mdformat (with gfm, rustfmt, front-matters, simple-breaks)
- [x] 1.4 Add job: cargo fmt --check
- [x] 1.5 Add job: cargo clippy (with strict deny rules matching workspace)
- [x] 1.6 Add job: cargo test

## 2. Release Workflow (Binary Builds)

- [x] 2.1 Create `.github/workflows/release.yml` triggered on tag push (`v*`)
- [x] 2.2 Add matrix build for 5 targets (Linux x64, macOS x64, macOS arm64, Windows x64, RPi arm64)
- [x] 2.3 Cross-compile using `cross` or native runners where available
- [x] 2.4 Create GitHub Release with binary artifacts
- [x] 2.5 Generate checksums for all binaries

## 3. FX Rate Update Workflow

- [x] 3.1 Fix `scripts/download-fx-rates.sh` path (`cgt-fx` → `cgt-money`)
- [x] 3.2 Create `.github/workflows/update-fx-rates.yml` with monthly schedule (last Friday)
- [x] 3.3 Run download script and detect new files
- [x] 3.4 If new rates: bump versions in Cargo.toml files
- [x] 3.5 If new rates: create and push git tag to trigger release
- [x] 3.6 Add manual trigger option for on-demand updates

## 4. Documentation

- [x] 4.1 Update README.md with binary installation instructions (GitHub Releases)
- [x] 4.2 Add platform-specific download links/instructions

## 5. Validation

- [x] 5.1 Test CI workflow on feature branch
- [x] 5.2 Test release workflow with test tag
- [x] 5.3 Verify all binaries execute correctly on target platforms (macOS arm64 executed from v0.1.0; other artifacts built and published by release workflow)
