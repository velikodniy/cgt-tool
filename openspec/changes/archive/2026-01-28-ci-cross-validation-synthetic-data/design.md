## Context

The project has a working cross-validation script (`scripts/cross-validate.py`) that compares cgt-tool output against two external UK CGT calculators:

- KapJI/capital-gains-calculator (Python) - invoked via `uvx cgt-calc`
- mattjgalloway/cgtcalc (Swift) - requires building from source

Currently this runs manually. The existing test fixtures (`tests/inputs/*.cgt`) provide good coverage of matching rules but lack comprehensive RSU vesting patterns that would exercise the Schwab converter with realistic data.

## Goals / Non-Goals

**Goals:**

- Automate cross-validation in CI without blocking PRs (separate workflow)
- Create synthetic test data that exercises all CGT matching rules
- Provide Schwab-format JSON fixtures for converter testing
- Cover RSU vesting edge cases observed in real broker exports

**Non-Goals:**

- Running cross-validation on every PR (too slow, external dependencies)
- Modifying the cross-validate.py script behavior
- Adding new external calculators beyond the existing two
- Testing actual personal financial data

## Decisions

### Decision 1: Separate Workflow File

**Choice:** Create `cross-validate.yml` separate from `test.yml`

**Rationale:** Cross-validation depends on external tools (Python/uv, Swift toolchain) and is slower than unit tests. Keeping it separate means:

- PRs aren't blocked by external tool failures
- Can run on different schedule (weekly vs. every push)
- Different runner requirements (macOS for Swift)

**Alternatives Considered:**

- Integrate into test.yml with `continue-on-error: true` - rejected because it clutters the main CI and still runs on every PR
- Make it a reusable workflow called from test.yml - adds complexity without benefit

### Decision 2: Single-Job Architecture

**Choice:** Use one `cross-validate` job on macos-latest that runs both calculators

**Rationale:**

- Single workflow job is simpler to operate and reason about
- macOS runner supports Swift toolchain needed for cgtcalc
- Python tooling and uv are available on the same runner

**Alternatives Considered:**

- Split into ubuntu + macOS jobs - faster but more complex to maintain
- Skip Swift/cgtcalc entirely - loses valuable cross-validation coverage

### Decision 3: Manual Trigger Only

**Choice:** Run via `workflow_dispatch` only

**Rationale:**

- Keeps CI costs controlled during early rollout
- Maintainers can run validation on-demand before releases

### Decision 4: Synthetic Data Structure

**Choice:** Create three files:

- `tests/inputs/SyntheticComplex.cgt` - the CGT DSL test fixture
- `tests/schwab/synthetic-awards.json` - Schwab awards format
- `tests/schwab/synthetic-transactions.json` - Schwab transactions format

**Rationale:**

- CGT file tests the calculator directly
- JSON files test the converter round-trip
- Keeping JSON in `tests/schwab/` matches the broker-specific organization

**Alternatives Considered:**

- Generate CGT from JSON automatically - adds complexity, harder to verify edge cases are correct
- Multiple smaller files - harder to test cross-year interactions

### Decision 5: Synthetic Data Content

**Choice:** 5 tax years, 3 tickers, all matching rules

**Design:**

- ACME (USD): RSU vesting pattern with multi-award same-day vests, sell-to-cover
- BETA (USD): Regular trading with Same Day, B&B, partial B&B scenarios
- GAMA (GBP): UK stock for multi-currency testing

**Edge Cases Covered:**

- Multi-award same-day vesting (6 awards vest same day)
- Same-day vest + immediate sell-to-cover
- FMV vs sale price discrepancy for cost basis
- Multiple sells same day at different prices
- Consecutive day selling (drip pattern)
- Tax year boundary (April 5/6)
- Exact 30-day B&B boundary
- Buy-sell-buy same day
- Stock split
- Capital return

### Decision 6: Fictional Company Names

**Choice:** Use ACME, BETA, GAMA as ticker symbols

**Rationale:**

- Clearly synthetic - no confusion with real data
- Memorable and easy to reference in documentation
- Avoids any accidental resemblance to real portfolios

## Risks / Trade-offs

**Risk: External calculator API changes**
→ Mitigation: Pin specific versions in workflow, update converter scripts when needed

**Risk: Swift toolchain setup on macOS runners**
→ Mitigation: Use GitHub's pre-installed Swift, add fallback to skip cgtcalc job if setup fails

**Risk: Synthetic data doesn't catch real-world edge cases**
→ Mitigation: Design based on observed patterns in actual broker exports (fedor example structure)

**Risk: Cross-validation discrepancies may be in external calculators, not cgt-tool**
→ Mitigation: Document known discrepancies, focus on £1+ differences as actionable

**Trade-off: macOS runner cost**
→ Acceptable for manual runs; schedule can be added later if needed
