# Test Verification Data Model

**Purpose:** Lightweight conceptual model for tracking test verification progress

**Note:** This is not a traditional software data model. It represents the verification workflow entities for planning and tracking purposes. Actual implementation may use markdown checklists, spreadsheets, or inline comments.

---

## Entities

### Test Case

Represents a single .cgt test file with associated expected output.

**Attributes:**

- `file_name`: String (e.g., "Simple.cgt")
- `category`: Enum
  - `SameDay`: Tests Same Day matching rule only
  - `BedAndBreakfast`: Tests B&B 30-day rule only
  - `Section104`: Tests Section 104 pooling only
  - `MultiRule`: Tests combination of multiple rules
  - `EdgeCase`: Tests special cases (splits, capital returns, unsorted, etc.)
- `complexity`: Enum
  - `Simple`: Few transactions (≤5), single tax year, straightforward matching
  - `Complex`: Many transactions (>5), multi-year, or special events (splits, capital returns)
- `has_header_comments`: Boolean
- `has_inline_comments`: Boolean
- `verification_status`: Enum
  - `NotStarted`: No verification performed
  - `InProgress`: Verification underway
  - `Verified`: Manual verification complete, matches expected output
  - `DiscrepancyFound`: Mismatch between manual calculation and expected output
- `cgtcalc_equivalent`: Optional<String>
  - Maps to cgtcalc test name (e.g., "Simple.txt")
  - `null` if no equivalent test in cgtcalc
- `requires_detailed_verification`: Boolean
  - `true` for 5+ priority tests needing detailed step-by-step calculations
  - `false` for tests requiring only header + simple verification

**Relationships:**

- Has many `VerificationNote`
- May have many `Discrepancy`
- References one optional cgtcalc test (external)

**Invariants:**

- If `verification_status == Verified`, must have `has_header_comments == true`
- If `requires_detailed_verification == true`, verification notes must include detailed calculations
- `cgtcalc_equivalent` name (without extension) should match `file_name` (without extension) if mapping exists

---

### Verification Note

Represents documentation within a .cgt file explaining calculations and verification.

**Attributes:**

- `test_file`: Reference to `TestCase`
- `line_number`: Optional<Integer>
  - Line number in .cgt file where comment appears
  - `null` for header comments (precede all transactions)
- `note_type`: Enum
  - `Header`: File header comment with metadata
  - `Inline`: Comment adjacent to specific transaction
  - `DetailedCalculation`: Step-by-step calculation for priority tests
- `content`: String
  - The actual comment text (excluding `#` prefix)
  - Multi-line comments stored as single string with newlines

**Relationships:**

- Belongs to one `TestCase`

**Invariants:**

- Header notes must have `line_number == null` or `line_number < first_transaction_line`
- Inline notes must reference specific transaction line
- DetailedCalculation notes only exist for tests with `requires_detailed_verification == true`

---

### Discrepancy

Represents a difference between manually calculated result and expected output.

**Attributes:**

- `test_file`: Reference to `TestCase`
- `description`: String
  - Human-readable description of the discrepancy
  - Example: "Total gain calculated as £520, expected £515 in .json"
- `our_result`: String
  - Our manually calculated gain/loss values
  - Format: "Gain: £X, Loss: £Y"
- `expected_result`: String
  - Expected values from .json file or cgtcalc
  - Same format as `our_result`
- `cgtcalc_result`: Optional<String>
  - Result from cgtcalc if different from our expected
  - `null` if cgtcalc matches our expectation
- `resolution`: String
  - Outcome after HMRC guidance consultation
  - Documents decision and action taken
- `root_cause`: Enum
  - `OurCodeBug`: Bug in crates/cgt-core/src/calculator.rs or parser.rs
  - `OurTestBug`: Incorrect expected values in .json file
  - `CgtcalcBug`: cgtcalc has incorrect implementation
  - `LegitDifference`: Both implementations valid, different edge case handling
  - `RoundingDifference`: Within tolerance (±1.0), acceptable
- `hmrc_reference`: String
  - HMRC guidance section used for resolution
  - Example: "CG51560 - Same Day Rule"

**Relationships:**

- Belongs to one `TestCase`
- May reference cgtcalc test result

**Invariants:**

- If `root_cause == OurCodeBug`, resolution must document code fix
- If `root_cause == OurTestBug`, resolution must include HMRC proof for correction
- If `root_cause == LegitDifference`, resolution must document chosen interpretation with HMRC justification
- `hmrc_reference` must be populated for all discrepancies

---

### Test Comparison

Maps cgtcalc tests to cgt-tool tests for comparison purposes.

**Attributes:**

- `cgtcalc_test_name`: String (e.g., "Simple.txt")
- `cgt_tool_test_name`: String (e.g., "Simple.cgt")
- `match_status`: Enum
  - `ExactMatch`: Test names match, both exist
  - `CgtToolOnly`: Test exists in cgt-tool but not cgtcalc
  - `CgtcalcOnly`: Test exists in cgtcalc but not cgt-tool (shouldn't occur based on research)
- `transaction_comparison`: String
  - Summary of transaction-level comparison
  - Notes on DSL syntax translation
  - Line order differences
- `result_comparison`: String
  - Comparison of expected outputs
  - Notes on format differences (.json vs .txt)
  - Any discrepancies in gain/loss values

**Relationships:**

- References one `TestCase` in cgt-tool
- References one test in cgtcalc (external)

**Invariants:**

- `match_status == ExactMatch` implies both test names identical (ignoring extension)
- For cgtcalc repository commit 896d914868, all 21 tests should have `match_status == ExactMatch`

---

## Verification Workflow States

### Test Case State Transitions

```
NotStarted
    ↓ (Begin verification)
InProgress
    ↓ (Complete manual calculation)
Verified  OR  DiscrepancyFound
              ↓ (Resolve discrepancy)
              Verified
```

### Completion Criteria

A `TestCase` is considered complete when:

1. `has_header_comments == true` (metadata added)
2. `has_inline_comments == true` OR `complexity == Simple` (complex tests need inline notes)
3. `verification_status == Verified` (manual calculation matches expected)
4. If `requires_detailed_verification == true`: DetailedCalculation notes exist
5. Test passes `cargo test` (automated test succeeds)

---

## Example Instances

### TestCase: Simple.cgt

```yaml
file_name: "Simple.cgt"
category: SameDay
complexity: Simple
has_header_comments: true  # After implementation
has_inline_comments: false  # Not required for simple test
verification_status: Verified  # After manual verification
cgtcalc_equivalent: "Simple.txt"
requires_detailed_verification: false
```

### TestCase: HMRCExample1.cgt

```yaml
file_name: "HMRCExample1.cgt"
category: MultiRule
complexity: Complex
has_header_comments: true
has_inline_comments: true
verification_status: Verified
cgtcalc_equivalent: "HMRCExample1.txt"
requires_detailed_verification: true  # Priority test
```

### VerificationNote: Header for HMRCExample1.cgt

```yaml
test_file: HMRCExample1.cgt
line_number: null  # Header comment
note_type: Header
content: |
  Test: HMRCExample1
  Purpose: Validates official HMRC Example 3 from HS284_Example_3_2020.pdf
  Rules Tested: Multiple (Section 104, Bed & Breakfast)
  Complexity: Complex
  Key Features: Multi-year (2014-2019), official HMRC example
  Expected Outcome: Gain calculation per HMRC worked example

  Verification Status: Verified
  Verified By: Manual verification 2025-12-08
  Verification Notes: See detailed verification below
```

### Discrepancy: Hypothetical example

```yaml
test_file: WithAssetEventsBB.cgt
description: "Calculated gain £1205, expected £1200 in .json"
our_result: "Gain: £1205, Loss: £0"
expected_result: "Gain: £1200, Loss: £0"
cgtcalc_result: "£1200"  # cgtcalc agrees with our .json
resolution: "Pool cost calculation error in calculator.rs line 245. Capital return should reduce pool cost before disposal, not after. Fixed per HMRC CG58620."
root_cause: OurCodeBug
hmrc_reference: "CG58620 - Capital Returns Treatment"
```

---

## Implementation Notes

**Actual Implementation Strategy:**

This data model serves as a conceptual framework. Actual tracking will use:

1. **Test file comments (.cgt files)**: Primary storage for verification notes
2. **Markdown checklist** (optional): Track verification progress across all tests
3. **Git commit messages**: Document discrepancy resolutions and code fixes
4. **research.md**: Store cgtcalc comparison mapping table

**No database or code implementation required** - this is a documentation/verification feature, not a software feature.

---

**Document Status:** Data model complete
**Last Updated:** 2025-12-08
