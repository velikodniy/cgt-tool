## 1. Setup

- [x] 1.1 Create Personal Access Token with `repo` scope at https://github.com/settings/tokens
- [x] 1.2 Add token as `TAP_REPO_TOKEN` secret in cgt-tool repo settings (Settings > Secrets and variables > Actions)

## 2. Implementation

- [x] 2.1 Create `.github/workflows/notify-tap.yml` workflow file
- [x] 2.2 Configure workflow to trigger on `release: [published]` events
- [x] 2.3 Add `repository-dispatch` step using `peter-evans/repository-dispatch@v3`
- [x] 2.4 Pass release tag via `client-payload`

## 3. Validation

- [x] 3.1 Verify workflow syntax with `gh workflow list` or Actions tab
- [x] 3.2 Test with manual trigger of tap workflow: `gh workflow run update-cgt-tool.yml -R velikodniy/homebrew-tap`
- [x] 3.3 Confirm end-to-end on next release (or test release)
