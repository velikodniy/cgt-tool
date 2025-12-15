# Design: RSU Acquisition Date Fix

## Context

When RSUs vest, there are typically two relevant dates:

1. **Vest/Lapse Date**: When vesting conditions are satisfied and employee becomes unconditionally entitled to shares
2. **Settlement Date**: When shares are deposited into brokerage account (T+2)

Schwab's transaction CSV shows `Stock Plan Activity` on the settlement date (or with "as of" notation). The awards CSV contains the vest/lapse date in the `Date` column of Lapse rows.

Currently, cgt-tool uses the settlement date from transactions. Capital-gains-calculator (the reference Python tool) uses the lapse date from awards.

## Goals

- Use vest date as CGT acquisition date per HMRC guidance
- Maintain backward compatibility for existing workflows
- Ensure FMV lookback still works correctly

## Non-Goals

- Support brokers without awards files
- Change how sales dates are determined

## Decisions

### Decision: Return vest date from AwardsData lookup

**What**: Modify `AwardsData::get_fmv()` to return both FMV and vest date in an `AwardLookup` struct.

**Why**: The awards file is already keyed by vest date. Returning this date alongside FMV allows the converter to use it as the acquisition date.

**Alternatives considered**:

1. Separate method `get_vest_date()` - Rejected: would require duplicate lookback logic
2. Store settlement→vest date mapping - Rejected: adds complexity without benefit since FMV lookup already finds the vest date

### Decision: Vest date takes priority over transaction date

**What**: When processing `Stock Plan Activity`, use the vest date from awards file as the BUY transaction date, ignoring the settlement date in the transaction row.

**Why**: HMRC guidance is clear that acquisition occurs when conditions are satisfied (vest), not when shares settle.

### Decision: 7-day lookback unchanged

**What**: Keep existing 7-day lookback from transaction date to find matching awards.

**Why**: Settlement is typically 2-3 days after vest, well within the 7-day window. No change needed.

## Risks / Trade-offs

| Risk                                      | Impact | Mitigation                                                |
| ----------------------------------------- | ------ | --------------------------------------------------------- |
| Different results from previous runs      | Medium | Document as intentional correction in changelog           |
| JSON awards format may not have vest date | Low    | JSON format uses EventDate which is already the vest date |

## Migration Plan

1. Update `AwardsData` struct and `get_fmv` method
2. Update Schwab converter to use returned vest date
3. Update tests to verify correct date usage
4. Document change in TAX_RULES.md

No data migration needed - this is a calculation fix.

## Open Questions

None - HMRC guidance is clear on this point.
