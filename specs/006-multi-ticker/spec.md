# Feature Specification: Multi-Ticker Support

**Feature Branch**: `006-multi-ticker`
**Created**: 2025-12-08
**Status**: Draft
**Input**: User description: "focus on multi-ticker support: currently `pool` is a single holding, fix this, add more tests for multi-tickers, the tests MUST contain manual calculations like other tests, you MUST be careful, you MUST review the existing code and refactor if needed to simplify it."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Calculate CGT for Portfolio with Multiple Stocks (Priority: P1)

A user with investments in multiple companies (e.g., AAPL, MSFT, GOOG) wants to calculate their Capital Gains Tax. Each stock should have its own Section 104 pool, and the system should correctly track and calculate gains/losses independently for each ticker.

**Why this priority**: This is the core functionality gap - the current system only supports a single ticker, which is fundamentally broken for real-world CGT calculations where users have diversified portfolios.

**Independent Test**: Can be fully tested by creating a .cgt file with transactions for multiple tickers and verifying each ticker's pool is tracked separately.

**Acceptance Scenarios**:

1. **Given** transactions for AAPL and MSFT on different dates, **When** calculating CGT, **Then** each ticker has its own Section 104 pool with correct quantities and costs.
2. **Given** a sale of AAPL shares, **When** calculating allowable cost, **Then** only the AAPL pool is used (not MSFT pool).
3. **Given** transactions for 3+ different tickers, **When** generating the report, **Then** holdings show all tickers with correct final quantities and costs.

---

### User Story 2 - Same Day Matching with Multiple Tickers (Priority: P2)

A user buys and sells different stocks on the same day. The Same Day matching rule should only match transactions of the same ticker.

**Why this priority**: Same Day is the first matching rule applied, and incorrect cross-ticker matching would cause wrong CGT calculations.

**Independent Test**: Can be tested by creating same-day buy/sell pairs for different tickers and verifying they don't cross-match.

**Acceptance Scenarios**:

1. **Given** same-day BUY AAPL and SELL MSFT, **When** applying Same Day rule, **Then** no match occurs (different tickers).
2. **Given** same-day BUY AAPL, SELL AAPL, BUY MSFT, SELL MSFT, **When** applying Same Day rule, **Then** AAPL matches AAPL and MSFT matches MSFT.

---

### User Story 3 - Bed & Breakfast Matching with Multiple Tickers (Priority: P2)

A user sells shares and repurchases within 30 days. The B&B rule should only match transactions of the same ticker.

**Why this priority**: B&B is the second matching rule and must respect ticker boundaries to comply with UK tax law.

**Independent Test**: Can be tested by creating B&B scenarios with different tickers and verifying no cross-ticker matching.

**Acceptance Scenarios**:

1. **Given** SELL AAPL on day 1, BUY MSFT on day 15, **When** applying B&B rule, **Then** no match occurs (different tickers).
2. **Given** SELL AAPL on day 1, BUY AAPL on day 15, BUY MSFT on day 10, **When** applying B&B rule, **Then** only AAPL sale matches AAPL buy.

---

### User Story 4 - Stock Splits/Consolidations Per Ticker (Priority: P3)

A user has multiple stocks and one undergoes a stock split. The split should only affect that specific ticker's pool.

**Why this priority**: Stock splits are less common but must be handled correctly per-ticker.

**Independent Test**: Can be tested by having a split for one ticker and verifying other tickers' pools are unaffected.

**Acceptance Scenarios**:

1. **Given** holdings in AAPL and MSFT, **When** AAPL has a 2:1 split, **Then** only AAPL quantity doubles, MSFT unchanged.
2. **Given** holdings in AAPL, MSFT, GOOG, **When** MSFT consolidates 1:10, **Then** only MSFT quantity divides, others unchanged.

---

### Edge Cases

- What happens when selling a ticker that has no prior acquisitions? System returns an error.
- How does the system handle mixed transactions (buys/sells of different tickers on same day)? Each ticker's transactions are matched independently.
- What if a ticker appears in transactions but has zero remaining holdings after all sales? The ticker may be omitted from final holdings or shown with zero quantity.
- What if user enters "aapl" and "AAPL" in different transactions? Both are normalized to "AAPL" and treated as the same ticker.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST maintain separate Section 104 pools for each unique ticker symbol.
- **FR-002**: Same Day matching MUST only match BUY and SELL transactions with identical ticker symbols.
- **FR-003**: Bed & Breakfast matching MUST only match transactions with identical ticker symbols.
- **FR-004**: Stock SPLIT operations MUST only affect the pool of the specified ticker.
- **FR-005**: Stock UNSPLIT operations MUST only affect the pool of the specified ticker.
- **FR-006**: The holdings output MUST list all tickers with non-zero holdings and their respective pool values.
- **FR-007**: System MUST return an error if attempting to sell shares of a ticker with no prior acquisitions.
- **FR-008**: System MUST return an error if attempting to sell more shares than available in a ticker's pool.
- **FR-009**: System MUST normalize ticker symbols to uppercase during parsing (case-insensitive matching).

### Key Entities

- **Section104Pool**: A per-ticker holding that tracks quantity and total cost basis. Each ticker has exactly one pool (or none if no acquisitions).
- **Transaction**: Buy/Sell/Split/Unsplit/CapReturn/Dividend operation with associated ticker symbol.
- **TaxReport**: Contains tax year summaries (disposals, gains, losses) and final holdings across all tickers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All existing single-ticker tests continue to pass without modification.
- **SC-002**: New multi-ticker tests demonstrate correct independent pool tracking (minimum 3 test cases with manual calculations).
- **SC-003**: Same Day and B&B rules correctly filter by ticker (verified by test cases showing no cross-ticker matching).
- **SC-004**: Holdings output correctly shows all tickers with their final pool state.
- **SC-005**: Code complexity is reduced or maintained (no increase in cyclomatic complexity).

## Clarifications

### Session 2025-12-08

- Q: Algorithm architecture for multi-ticker processing? → A: Group transactions by ticker first, process each ticker independently, then merge results.
- Q: Are ticker names case-sensitive? → A: Case-insensitive with uppercase normalization (aapl → AAPL during parsing).

## Assumptions

- All transactions in a single .cgt file may contain multiple ticker symbols.
- Ticker symbols are case-insensitive; input is normalized to uppercase during parsing (e.g., "aapl" → "AAPL").
- Each ticker's Section 104 pool is independent - there is no cross-ticker cost basis sharing.
- Algorithm follows "split-process-merge" pattern: transactions are grouped by ticker, each group is processed through the existing 3-pass matching logic (Same Day → B&B → Section 104), then results are merged into the final report.
