# Feature Specification: Test Suite Validation and UK CGT Tax Rules Documentation

**Feature Branch**: `004-test-validation`
**Created**: 2025-12-08
**Status**: Complete
**Input**: User description: "Let's focus on the tests. - Do research about the tax rules and create a document in the root of the project with your findings in a brief form. - Download tests from https://github.com/mattjgalloway/cgtcalc and compare them with the tests in this repo - Add attribution to README mentioning that we used the tests - The tests should be the same because we trust cgtcalc. Exceptions: the line order (in ours it's reversed) and the DSL (we use a bit different syntax) - Verify the tests manually by thinking carefully at each case and doing calculations - The tests MUST be the source of truth in the future. It's EXTREMELY IMPORTANT. - Support the cases the cgtcalc repo doesn't support - If needed correct the code, but you MUST make careful verification that the numbers are correct."

## Clarifications

### Session 2025-12-08

- Q: Where should manual verification calculations be documented? → A: In .cgt test files as comments (inline with transactions)
- Q: When our calculation differs from cgtcalc, what determines which is correct? → A: HMRC official guidance is authoritative; both implementations adjusted if needed
- Q: What should the tax rules documentation file be named? → A: TAX_RULES.md
- Q: How should cgtcalc test files be obtained? → A: Manual download with documentation of source commit/date (tests used for comparison only, not kept in repo)
- Q: Should all test cases have detailed manual verification, or just representative samples? → A: All tests verified; only complex/representative cases need detailed step-by-step documentation

## User Scenarios & Testing *(mandatory)*

### User Story 1 - UK CGT Tax Rules Documentation (Priority: P1)

Developers and maintainers need authoritative documentation of UK Capital Gains Tax rules to understand the calculation logic and verify test cases.

**Why this priority**: Without proper documentation of the tax rules, it's impossible to verify that calculations are correct or to understand why test cases produce specific results. This is foundational for all other validation work.

**Independent Test**: Can be fully tested by reading the documentation and confirming it covers Same Day, Bed & Breakfast (30-day rule), and Section 104 pooling rules with concrete examples and references to HMRC guidance.

**Acceptance Scenarios**:

1. **Given** a developer joins the project, **When** they read the tax rules document, **Then** they understand how Same Day matching works with specific examples
2. **Given** a maintainer reviews a bug report, **When** they consult the tax rules document, **Then** they can determine if the reported behavior is correct per HMRC rules
3. **Given** the tax rules document exists, **When** someone reads it, **Then** they find clear explanations of Bed & Breakfast (30-day) rule with edge cases
4. **Given** the tax rules document exists, **When** someone reads it, **Then** they find Section 104 pooling calculations with worked examples

---

### User Story 2 - Test Case Validation Against cgtcalc Reference (Priority: P1)

Developers need assurance that the test suite matches the trusted cgtcalc reference implementation to ensure calculation accuracy.

**Why this priority**: The cgtcalc project is a trusted reference implementation. Aligning our tests with theirs provides confidence in correctness and catches any discrepancies in our implementation.

**Independent Test**: Can be fully tested by downloading cgtcalc tests, comparing them with our tests (accounting for line order and DSL differences), and confirming all test cases produce equivalent results.

**Acceptance Scenarios**:

1. **Given** cgtcalc test files downloaded, **When** comparing with our test files, **Then** each cgtcalc test case has a corresponding test in our suite
2. **Given** test files from both repos, **When** accounting for line order (reversed) and DSL syntax differences, **Then** the transaction sequences are equivalent
3. **Given** a test case from cgtcalc, **When** run through our calculator, **Then** the gain/loss totals match within acceptable rounding tolerance
4. **Given** discrepancies are found, **When** investigating, **Then** root cause is identified and either our code or test expectations are corrected
5. **Given** test suite validation complete, **When** reviewing README, **Then** attribution to cgtcalc is clearly stated

---

### User Story 3 - Manual Test Case Verification (Priority: P2)

Maintainers need manual verification of test calculations to establish tests as the authoritative source of truth.

**Why this priority**: Automated test comparison isn't enough - we need human verification that the expected results are actually correct per UK tax law. This ensures tests are trustworthy for future development.

**Independent Test**: Can be fully tested by selecting representative test cases, manually calculating expected gains/losses using the tax rules document, and confirming they match test expectations.

**Acceptance Scenarios**:

1. **Given** a Same Day matching test case, **When** manually calculating the gain/loss, **Then** the calculation matches the expected result in the test
2. **Given** a Bed & Breakfast test case, **When** manually walking through the 30-day rule, **Then** the matches and gain/loss are verified correct
3. **Given** a Section 104 pooling test case, **When** manually calculating the pool average cost, **Then** the final gain/loss is verified correct
4. **Given** a multi-year test case, **When** manually verifying tax year boundaries, **Then** gains/losses are attributed to the correct tax year
5. **Given** edge cases (splits, capital returns, partial sales), **When** manually verifying calculations, **Then** all special cases are handled correctly

---

### User Story 4 - Self-Documenting Test Files (Priority: P2)

Developers need test files to be self-documenting with comments explaining what each test case validates.

**Why this priority**: Test files are the primary way developers understand what scenarios are covered. Comments in .cgt files make tests immediately understandable without needing external documentation.

**Independent Test**: Can be fully tested by reviewing each .cgt test file and confirming it has comments explaining the test scenario, what rule it validates, and expected behavior.

**Acceptance Scenarios**:

1. **Given** a .cgt test file, **When** opening it, **Then** the first lines contain comments explaining the test purpose
2. **Given** a complex test case, **When** reading the .cgt file, **Then** inline comments explain key transactions and expected matching
3. **Given** an edge case test, **When** reviewing comments, **Then** the specific edge case behavior being validated is clear
4. **Given** a multi-step test, **When** reading comments, **Then** the expected calculation flow (Same Day → B&B → Section 104) is documented

---

### User Story 5 - Extended Test Coverage (Priority: P3)

Developers need test coverage for cases not covered by cgtcalc to ensure comprehensive validation.

**Why this priority**: While cgtcalc provides excellent baseline coverage, there may be edge cases or features specific to this implementation that need additional tests. This ensures complete coverage.

**Independent Test**: Can be fully tested by identifying gaps in cgtcalc coverage, creating new test cases for those scenarios, and verifying they pass with correct calculations.

**Acceptance Scenarios**:

1. **Given** the cgtcalc test suite, **When** analyzing coverage, **Then** gaps in edge case coverage are identified
2. **Given** identified gaps, **When** creating new test cases, **Then** they cover scenarios not in cgtcalc (e.g., specific DSL features, complex transaction patterns)
3. **Given** new test cases, **When** manually verified, **Then** expected results are confirmed correct per UK tax rules
4. **Given** all test cases (cgtcalc + extended), **When** running full test suite, **Then** 100% pass with verified correct results

---

### Edge Cases

- When cgtcalc test has a different expected result than our calculation, HMRC official guidance is the authoritative source; both implementations may need adjustment
- How to handle rounding differences between implementations?
- What if cgtcalc test data format is incompatible with our DSL?
- How to verify calculations for complex multi-year scenarios with carried losses?
- What if HMRC tax rules have changed since cgtcalc tests were created?
- How to handle test cases with transactions that span multiple tax years?
- What if there are ambiguous scenarios where tax rules interpretation varies?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST have a document named TAX_RULES.md in the project root explaining UK CGT matching rules (Same Day, Bed & Breakfast, Section 104)
- **FR-002**: Tax rules document MUST include concrete examples with calculations for each matching rule
- **FR-003**: Tax rules document MUST reference official HMRC guidance where applicable
- **FR-004**: System MUST manually download test cases from the cgtcalc repository (https://github.com/mattjgalloway/cgtcalc) for one-time comparison, documenting source commit/date
- **FR-005**: System MUST compare cgtcalc test cases with existing tests, accounting for line order differences (reversed in our implementation)
- **FR-006**: System MUST compare cgtcalc test cases with existing tests, accounting for DSL syntax differences
- **FR-007**: README MUST include attribution to cgtcalc project for test case inspiration
- **FR-008**: Each .cgt test file MUST contain comments explaining what the test validates
- **FR-009**: Test file comments MUST describe the scenario, which tax rule is being tested, and expected behavior
- **FR-010**: Complex test cases MUST have inline comments explaining key transactions and matching logic
- **FR-011**: All test cases MUST be manually verified by calculating expected gain/loss using the documented tax rules
- **FR-012**: Manual verification calculations MUST be documented as comments in .cgt test files; complex/representative cases MUST have detailed step-by-step working
- **FR-013**: System MUST identify any discrepancies between our calculations and cgtcalc expected results
- **FR-014**: For each discrepancy, system MUST determine root cause using HMRC official guidance as the authoritative source; both implementations may require adjustment
- **FR-015**: System MUST correct production code if bugs are found during verification
- **FR-016**: Code corrections MUST be verified to produce mathematically correct results per UK tax law and HMRC guidance
- **FR-017**: System MUST identify test scenarios not covered by cgtcalc
- **FR-018**: System MUST add test cases for scenarios beyond cgtcalc coverage where appropriate
- **FR-019**: All test cases MUST be established as the source of truth for future development
- **FR-020**: Test suite MUST include verification that total gains and losses match expected values within acceptable rounding tolerance

### Key Entities

- **Tax Rule**: Represents one of the three UK CGT matching rules (Same Day, Bed & Breakfast, Section 104) with explanation and examples
- **Test Case**: A .cgt input file paired with .json expected output representing a specific tax scenario
- **Test Comment**: Documentation within a .cgt file explaining the test purpose, scenario, and expected behavior
- **Test Comparison**: Mapping between a cgtcalc test case and our equivalent test, documenting differences
- **Manual Verification**: Documentation of hand-calculated expected results for a test case with step-by-step working
- **Discrepancy**: A difference between our calculation and cgtcalc expected result, with root cause analysis

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: TAX_RULES.md document exists covering all three matching rules with at least 2 examples each
- **SC-002**: All cgtcalc test cases are manually downloaded, compared with our test suite, with source commit/date documented
- **SC-003**: 100% of .cgt test files contain comments explaining the test scenario
- **SC-004**: 100% of test cases are manually verified with calculations documented as comments in .cgt files
- **SC-005**: Any discrepancies between our results and cgtcalc are investigated and resolved using HMRC guidance as authority
- **SC-006**: All test cases pass with verified correct results within 1.0 rounding tolerance
- **SC-007**: README includes attribution to cgtcalc project
- **SC-008**: Test suite serves as the authoritative source of truth for future development
- **SC-009**: At least 5 complex/representative test cases from different categories (Same Day, B&B, Section 104, multi-year, edge cases) have detailed step-by-step verification documentation in comments

## Assumptions

- cgtcalc repository is publicly accessible and test files are in a parsable format
- cgtcalc test cases use similar transaction format that can be converted to our DSL
- Our "reversed line order" refers to chronological vs reverse-chronological ordering
- Acceptable rounding tolerance is ±1.0 (matching current test implementation)
- Manual verification will be documented as comments within .cgt test files
- "Source of truth" means tests define correct behavior and code must match tests (not vice versa)
- HMRC tax rules referenced in cgtcalc are current and haven't changed materially
- Comments in .cgt files use the # character syntax already supported by the parser
- cgtcalc tests are downloaded manually once for comparison; they are not kept in the repository long-term

## Out of Scope

- Automated conversion tools to transform cgtcalc test format to our DSL
- Maintaining cgtcalc tests in the repository (used for one-time comparison only)
- Performance testing or optimization of calculator
- Adding new DSL features beyond current capabilities
- Changing the DSL syntax to match cgtcalc exactly
- Creating a test harness that runs cgtcalc tests directly
- Validating test cases against real HMRC calculations or professional tax software
- Updating HMRC tax rule documentation if laws change in the future
