# Implementation Plan: Test Suite Validation and UK CGT Tax Rules Documentation

**Branch**: `004-test-validation` | **Date**: 2025-12-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-test-validation/spec.md`

## Summary

This feature establishes comprehensive test validation and documentation for the CGT calculation engine. Primary deliverables: (1) TAX_RULES.md document explaining UK CGT matching rules with worked examples, (2) verification and documentation of all test cases with inline comments in .cgt files, (3) comparison with cgtcalc reference implementation, (4) manual verification of calculations against HMRC guidance, and (5) README attribution. This is primarily a documentation, research, and verification task rather than code implementation.

## Technical Context

**Language/Version**: Rust 2024 edition (existing)
**Primary Dependencies**: No new dependencies (documentation and test verification only)
**Storage**: Filesystem (.cgt test files, .json expected outputs, markdown documentation)
**Testing**: cargo test (existing), manual verification against HMRC rules
**Target Platform**: Documentation (markdown), test files (.cgt with comments)
**Project Type**: Rust workspace (crates/cgt-core, crates/cgt-cli)
**Performance Goals**: N/A (documentation task)
**Constraints**: Must not modify existing test expectations without proof of incorrectness per HMRC guidance
**Scale/Scope**: 22 existing .cgt test files, cgtcalc repository download for comparison

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle III: Modern Testing Standards (NON-NEGOTIABLE)

**Status**: ✅ PASS

- **Preservation Rules**: This feature explicitly enforces "never change previous tests without proving incorrect" - aligned with constitution
- **Tests as Source of Truth**: FR-019 establishes tests as authoritative, matching constitution principle
- **Verification Requirement**: Manual verification against HMRC guidance ensures test correctness before any changes

**Key Safeguards**:

- FR-014: Discrepancies resolved using HMRC official guidance as authority
- FR-016: Code corrections must be verified against UK tax law
- SC-004: 100% of test cases manually verified
- Constitution Principle VI explicitly requires domain verification

### Principle VI: Domain Mastery & Verification

**Status**: ✅ PASS

- **Research Phase**: Phase 0 includes comprehensive HMRC guidance research
- **Manual Verification**: FR-011, FR-012 require hand-calculation verification
- **Documentation**: TAX_RULES.md provides canonical domain knowledge
- **Authority Source**: HMRC guidance explicitly designated as authoritative

### Principle I: Deep Modules & Simplicity

**Status**: ✅ PASS (No New Complexity)

- This feature adds documentation and test comments only
- No new modules, interfaces, or abstractions introduced
- Existing codebase architecture unchanged

**Summary**: No constitution violations. Feature actively reinforces Principles III and VI by establishing rigorous test verification process.

## Project Structure

### Documentation (this feature)

```text
specs/004-test-validation/
├── plan.md              # This file
├── research.md          # Phase 0: HMRC tax rules research, cgtcalc analysis
├── data-model.md        # Phase 1: Test verification workflow (lightweight)
├── quickstart.md        # Phase 1: Guide for manual verification process
└── tasks.md             # Phase 2: Implementation checklist (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── cgt-core/
│   ├── src/              # No changes to production code (unless bugs found)
│   │   ├── calculator.rs
│   │   ├── parser.rs
│   │   └── models.rs
│   └── tests/
│       ├── parser_tests.rs
│       └── matching_tests.rs
└── cgt-cli/
    └── src/              # No changes expected

tests/data/               # MODIFIED: Add comments to all .cgt files
├── Simple.cgt            # ← Add header comments + verification notes
├── GainsAndLosses.cgt    # ← Add header comments + verification notes
├── SameDayMerge.cgt      # ← Add header comments + detailed verification
├── HMRCExample1.cgt      # ← Add header comments + detailed verification
└── [18 more .cgt files]  # ← Add comments to all

TAX_RULES.md              # NEW: UK CGT matching rules documentation (project root)

README.md                 # MODIFIED: Add cgtcalc attribution section

.gitignore                # MODIFIED: Ensure cgtcalc download directory ignored (if needed)
```

**Structure Decision**: Single Rust workspace project (existing). This feature primarily modifies documentation and test file comments, with potential production code fixes if bugs discovered during verification. No new directories or architectural changes.

## Complexity Tracking

> No constitution violations detected - this section intentionally left empty.

---

## Phase 0: Research & Domain Understanding

### Objectives

1. Research UK CGT tax matching rules from authoritative HMRC sources
2. Download and analyze cgtcalc test suite for comparison
3. Understand current test coverage and identify gaps
4. Document findings to inform verification workflow

### Research Tasks

#### R1: HMRC Tax Rules Research

**Goal**: Understand the three UK CGT matching rules with sufficient depth to manually verify calculations.

**Sources**:

- HMRC Capital Gains Manual (CG51500-CG51600 for share matching)
- HMRC guidance on Same Day rule
- HMRC guidance on Bed & Breakfast rule (30-day rule)
- HMRC guidance on Section 104 pooling

**Deliverable**: Section in research.md covering:

- Same Day Rule: Definition, examples, edge cases
- Bed & Breakfast Rule: 30-day matching, forward vs backward matching, examples
- Section 104 Pooling: Average cost calculation, pool maintenance, examples
- Tax year boundaries (April 6 to April 5)
- Treatment of expenses, capital returns, stock splits
- Carried losses across tax years

#### R2: cgtcalc Repository Analysis

**Goal**: Download cgtcalc tests and understand their structure for comparison.

**Actions**:

- Clone or download cgtcalc repository from https://github.com/mattjgalloway/cgtcalc
- Document commit hash and date for reproducibility
- Identify test file locations (likely `tests/` or similar directory)
- Analyze test file format and DSL syntax
- Map cgtcalc tests to our existing test suite

**Deliverable**: Section in research.md covering:

- cgtcalc repository commit hash and download date
- Test file locations and naming conventions
- DSL syntax differences (our syntax vs cgtcalc)
- Line order differences (our reversed chronological vs their chronological)
- Complete mapping table: cgtcalc test name → our test name
- Gaps: Tests in cgtcalc not in our suite, tests in our suite not in cgtcalc

#### R3: Current Test Suite Audit

**Goal**: Understand what our current 22 test files cover.

**Actions**:

- List all 22 .cgt test files
- Categorize by rule type (Same Day, B&B, Section 104, multi-rule, edge cases)
- Identify which tests are complex (multi-year, splits, capital returns)
- Review current test documentation (if any)

**Deliverable**: Section in research.md with:

- Complete test inventory with categorization
- Complexity assessment (simple vs complex)
- Current documentation status (which files have comments)
- Priority list for detailed verification (5+ complex/representative cases per SC-009)

#### R4: Verification Workflow Design

**Goal**: Define the manual verification process.

**Decisions Needed**:

- Comment format for .cgt files (header structure, inline notes)
- Level of detail for simple vs complex tests
- Template for detailed step-by-step verification (for 5+ representative cases)
- Process for discrepancy investigation

**Deliverable**: Section in research.md with:

- Comment template for .cgt file headers
- Comment template for inline verification notes
- Template for detailed step-by-step calculations
- Discrepancy resolution workflow (consult HMRC → determine correctness → update code or test)

### Research Output: research.md

**File**: `/Users/vadim/Projects/cgt-tool/specs/004-test-validation/research.md`

**Sections**:

1. UK CGT Matching Rules (from R1)
2. cgtcalc Comparison Analysis (from R2)
3. Current Test Suite Inventory (from R3)
4. Verification Workflow & Templates (from R4)
5. Open Questions (if any)

---

## Phase 1: Design & Documentation Structure

### Objectives

1. Create TAX_RULES.md with HMRC rules and examples
2. Define data model for test verification tracking (lightweight)
3. Create quickstart guide for manual verification process
4. Update agent context with domain knowledge

### Design Tasks

#### D1: TAX_RULES.md Document

**Location**: `/Users/vadim/Projects/cgt-tool/TAX_RULES.md`

**Structure**:

```markdown
# UK Capital Gains Tax Calculation Rules

## Overview
[Brief introduction to UK CGT and share matching]

## Rule 1: Same Day Matching
[Definition, HMRC reference]
### Example 1: Basic Same Day
[Concrete example with numbers]
### Example 2: Same Day Edge Case
[e.g., multiple transactions same day]

## Rule 2: Bed & Breakfast (30-Day Rule)
[Definition, HMRC reference]
### Example 1: Basic B&B Forward
[Purchase within 30 days after sale]
### Example 2: B&B with Section 104
[Interaction between rules]

## Rule 3: Section 104 Pooling
[Definition, average cost calculation]
### Example 1: Simple Pool
[Buy, buy, sell with pool calculation]
### Example 2: Multi-Year Pool
[Pool across tax years]

## Special Cases
### Stock Splits
### Capital Returns
### Carried Losses

## Tax Year Boundaries
[April 6 to April 5, examples]

## References
[Links to HMRC guidance]
```

**Source**: R1 research findings + examples from our test suite

#### D2: Data Model for Verification Tracking

**File**: `/Users/vadim/Projects/cgt-tool/specs/004-test-validation/data-model.md`

**Purpose**: Lightweight model to track verification status (not a traditional data model)

**Entities**:

```markdown
# Test Verification Data Model

## Test Case
- **file_name**: String (e.g., "Simple.cgt")
- **category**: Enum (SameDay, BedAndBreakfast, Section104, MultiRule, EdgeCase)
- **complexity**: Enum (Simple, Complex)
- **has_comments**: Boolean
- **verification_status**: Enum (NotStarted, InProgress, Verified, DiscrepancyFound)
- **cgtcalc_equivalent**: Optional<String> (mapped test name)
- **requires_detailed_verification**: Boolean (one of 5+ representative cases)

## Verification Note
- **test_file**: Reference to Test Case
- **line_number**: Optional<Integer>
- **note_type**: Enum (Header, Inline, DetailedCalculation)
- **content**: String (the actual comment text)

## Discrepancy
- **test_file**: Reference to Test Case
- **description**: String
- **our_result**: String (gain/loss values)
- **cgtcalc_result**: String
- **resolution**: String (outcome after consulting HMRC)
- **root_cause**: Enum (OurCodeBug, OurTestBug, CgtcalcBug, LegitDifference)
```

**Note**: This is a conceptual model for planning. Actual tracking may be in a simple markdown checklist or spreadsheet.

#### D3: Quickstart Guide

**File**: `/Users/vadim/Projects/cgt-tool/specs/004-test-validation/quickstart.md`

**Purpose**: Step-by-step guide for performing manual verification

**Content**:

```markdown
# Test Verification Quickstart Guide

## Prerequisites
- TAX_RULES.md document (read and understand)
- Calculator or spreadsheet
- HMRC guidance references

## Verification Process

### For All Tests (Simple Verification)
1. Open .cgt test file
2. Read transactions top to bottom
3. Identify expected matching rule (Same Day, B&B, S104)
4. Verify rule application makes sense
5. Add header comment explaining test purpose
6. Add inline comments for key transactions

### For Complex/Representative Tests (Detailed Verification)
1. Choose one of 5+ required representative tests
2. Create detailed calculation workspace (paper/spreadsheet)
3. Apply matching rules step by step:
   - Same Day: Match sells with buys same date
   - B&B: Check 30-day window forward from sale
   - Section 104: Calculate pool average cost
4. Document each step in .cgt file comments
5. Compare final gain/loss with .json expected output
6. If mismatch, investigate using HMRC guidance

### Discrepancy Resolution
1. Document discrepancy (our result vs expected)
2. Consult HMRC guidance for authoritative interpretation
3. Determine root cause:
   - Our code bug → fix calculator
   - Our test bug → update .json expectation (with proof)
   - Legitimate difference → document rationale
4. Verify fix produces correct result

## Comment Templates
[Include templates from research.md R4]
```

#### D4: Update Agent Context

**Action**: Run `.specify/scripts/bash/update-agent-context.sh claude`

**Updates**:

- Add "UK CGT Tax Rules" to domain knowledge
- Reference TAX_RULES.md location
- Note test verification workflow
- No new technology (documentation only)

### Design Outputs

1. `TAX_RULES.md` (project root)
2. `data-model.md` (specs/004-test-validation/)
3. `quickstart.md` (specs/004-test-validation/)
4. Updated `AGENTS.md` with domain context

---

## Phase 2: Task Breakdown (Created by /speckit.tasks command)

**Note**: This section is a placeholder. The `/speckit.tasks` command will generate `tasks.md` with detailed implementation tasks.

**Expected Task Categories**:

1. **Documentation Tasks**

   - Write TAX_RULES.md (Same Day, B&B, Section 104 sections)
   - Add README attribution to cgtcalc
   - Create verification templates

2. **cgtcalc Comparison Tasks**

   - Download cgtcalc repository
   - Document commit hash
   - Map tests to our suite
   - Identify gaps

3. **Test Comment Tasks**

   - Add header comments to all 22 .cgt files
   - Add inline comments for complex transactions
   - Write detailed verification for 5+ representative cases

4. **Verification Tasks**

   - Manually verify each test calculation
   - Document discrepancies
   - Investigate using HMRC guidance
   - Fix code bugs (if found)

5. **Quality Assurance**

   - Verify all tests pass (cargo test)
   - Review comment quality
   - Ensure SC-001 through SC-009 met

---

## Success Criteria Mapping

| Success Criterion                                          | Verification Method                                               |
| ---------------------------------------------------------- | ----------------------------------------------------------------- |
| SC-001: TAX_RULES.md exists with 3 rules, 2+ examples each | File exists, manual review of structure                           |
| SC-002: cgtcalc tests downloaded and compared              | Download logged with commit hash, comparison table in research.md |
| SC-003: 100% .cgt files have scenario comments             | Scan all 22 files for header comments                             |
| SC-004: 100% tests manually verified with comments         | Each .cgt has verification notes                                  |
| SC-005: Discrepancies resolved using HMRC guidance         | Discrepancy log shows HMRC-based resolution                       |
| SC-006: All tests pass within 1.0 tolerance                | `cargo test` passes                                               |
| SC-007: README includes cgtcalc attribution                | README section exists                                             |
| SC-008: Tests are source of truth                          | No test changes without HMRC proof                                |
| SC-009: 5+ complex cases have detailed verification        | Identify and verify 5+ detailed cases in comments                 |

---

## Risks & Mitigations

| Risk                                  | Impact                          | Mitigation                                                                                             |
| ------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------ |
| HMRC rules ambiguous or unclear       | Cannot verify tests confidently | Consult multiple HMRC sources, document ambiguity, seek expert input if needed                         |
| cgtcalc uses different interpretation | Conflicts in expected results   | HMRC guidance as authority, document rationale for differences                                         |
| Bugs found in our calculator          | Code changes required           | Follow constitution: fix code with proof, verify against HMRC, do not change tests to match buggy code |
| Time-intensive manual verification    | Delayed completion              | Prioritize 5+ representative cases for detailed verification, simpler verification for remaining tests |
| Test file format incompatibility      | Cannot compare with cgtcalc     | Manual translation with documented assumptions                                                         |

---

## Dependencies

**External**:

- HMRC Capital Gains Manual (online documentation)
- cgtcalc repository (GitHub, public)

**Internal**:

- Existing test suite (22 .cgt files, .json expected outputs)
- Existing calculator code (crates/cgt-core/src/calculator.rs)
- Existing parser (supports # comments)

**Blockers**:

- None (all dependencies accessible)

---

## Timeline Estimate

**Note**: This is documentation and verification work, not code implementation.

- **Phase 0 (Research)**: 2-3 hours

  - HMRC rules research: 1-1.5 hours
  - cgtcalc download and analysis: 0.5-1 hour
  - Test audit: 0.5 hour

- **Phase 1 (Documentation)**: 2-3 hours

  - TAX_RULES.md writing: 1.5-2 hours
  - Templates and guides: 0.5-1 hour

- **Implementation (Manual Verification)**: 6-8 hours

  - Header comments for 22 files: 1-2 hours
  - Simple verification (17 files): 2-3 hours
  - Detailed verification (5 files): 3-4 hours
  - Discrepancy resolution: Variable (0-2 hours)

**Total Estimate**: 10-14 hours

---

## Notes

1. **No Code Changes Expected**: This is primarily documentation and verification. Code changes only if bugs found.
2. **Constitution Alignment**: Feature actively enforces Principle III (test preservation) and Principle VI (domain mastery).
3. **HMRC Authority**: All verification decisions based on official HMRC guidance, not implementation consensus.
4. **Test Preservation**: Existing test expectations preserved unless proven incorrect by HMRC guidance.
5. **Comment Format**: Using existing # syntax, no parser changes needed.

---

**Plan Status**: ✅ Ready for Phase 0 (Research)

**Next Command**: `/speckit.tasks` (after Phase 0 and Phase 1 completion)
