# Change: Add GitHub Actions CI/CD Pipelines

## Why

The project lacks automated CI/CD, requiring manual verification before commits. Automated pipelines will ensure code quality, enable cross-platform binary releases, and keep FX rates up-to-date automatically.

## What Changes

- Add CI workflow for linting and testing on every push/PR
- Add release workflow for building binaries when tags are pushed
- Add scheduled workflow to update HMRC FX rates monthly
- Fix outdated path in `download-fx-rates.sh` (references `cgt-fx` but rates are in `cgt-money`)

## Impact

- Affected specs: New `ci-cd` capability
- Affected code:
  - `.github/workflows/ci.yml` (new)
  - `.github/workflows/release.yml` (new)
  - `.github/workflows/update-fx-rates.yml` (new)
  - `scripts/download-fx-rates.sh` (path fix)

## Design Decisions

### Schedule for FX Rate Updates

HMRC publishes rates on the **penultimate Thursday of every month**. The workflow will run on the **last Friday of each month** (day after potential publication) to ensure rates are captured reliably.

### Target Platforms

| Platform            | Target Triple               | Notes            |
| ------------------- | --------------------------- | ---------------- |
| Linux x86_64        | `x86_64-unknown-linux-gnu`  | Standard Linux   |
| macOS Intel         | `x86_64-apple-darwin`       | Intel Macs       |
| macOS Apple Silicon | `aarch64-apple-darwin`      | M1/M2/M3 Macs    |
| Windows x86_64      | `x86_64-pc-windows-msvc`    | Standard Windows |
| Raspberry Pi        | `aarch64-unknown-linux-gnu` | RPi 4/5 (64-bit) |

### Release Automation

When new FX rates are detected:

1. Bump minor version in `crates/cgt-money/Cargo.toml`
2. Bump minor version in `crates/cgt-cli/Cargo.toml`
3. Create git tag and push
4. This triggers the release workflow automatically
