<!--
SYNC IMPACT REPORT
Version: 1.0.0 -> 1.1.0
Changes:
- Modified Principle III (Modern Testing Standards) to explicitly forbid removing or modifying tests without proof.
- Added Principle VI (Domain Mastery & Verification).
- Added "Commit Discipline" to Development Workflow.
- Templates Status:
  - .specify/templates/plan-template.md: ✅ Compatible.
  - .specify/templates/spec-template.md: ✅ Compatible.
  - .specify/templates/tasks-template.md: ✅ Compatible.
  - .specify/templates/checklist-template.md: ✅ Compatible.
-->

# CGT Tool Constitution

## Core Principles

### I. Deep Modules & Simplicity

Complexity is anything that makes software hard to understand or modify. We adhere to "A Philosophy of Software Design" by John Ousterhout. Modules must be "deep"—providing powerful functionality through simple, abstract interfaces. Information hiding is paramount; implementation details must remain internal. If a module exposes its internal complexity, it is a design failure.

### II. Safety & Robustness

Code must be safe and modularized by default. Design systems to isolate failures (bulkheading). Prefer immutable data structures and strict typing to prevent runtime errors. Error handling must be explicit, graceful, and actionable—never fail silently or leave the system in an undefined state.

### III. Modern Testing Standards (NON-NEGOTIABLE)

Testing is a primary design activity, not an afterthought. Modern testing approaches (like TDD) are mandatory. Tests must be comprehensive, readable, and maintainable, serving as living documentation of system behavior.

**Preservation Rules:**

- **Never remove existing tests.**
- **Never change previous tests** without explicitly proving that they are incorrect. Changing a test to make code pass is an extremely bad practice.

A feature is not complete until it is fully tested and automated.

### IV. User Experience Consistency

User interaction must be consistent, predictable, and polished. Whether CLI or GUI, the interface must respect modern design principles. clear feedback, actionable error messages, and intuitive workflows are required. Do not burden the user with unnecessary complexity.

### V. Performance & Efficiency

Performance is a core feature. The system must respect user resources (CPU, RAM, Battery). Design for efficiency in critical paths. Measure and profile before optimizing, but architect to avoid inherent bottlenecks.

### VI. Domain Mastery & Verification

Complete and clear understanding of domain requirements is a prerequisite for implementation. Do not rely on assumptions. Verify implementations against domain rules, using manual computations to validate test expectations if necessary.

## Architectural Standards

### Technology Stack & Modernity

Use the best modern approaches and tools available for the task. Do not cling to legacy patterns if superior modern alternatives exist. Ensure all dependencies are actively maintained and secure.

### Modularity & Extension

The codebase must be clean, readable, and extendable. Adhere to high cohesion and low coupling. New features should be additive and not require extensive modification of existing stable code (Open/Closed Principle).

## Development Workflow

### Code Quality Gates

All code changes must pass strict quality gates:

1. **Automated Tests**: 100% pass rate required.
2. **Linting/Formatting**: Zero tolerance for style violations.
3. **Review**: Peer review must focus on simplicity, readability, and architectural alignment.

### Commit Discipline

- **Logical Atomicity**: Make logical commits. Never commit multiple logical changes altogether (e.g., adding a library then improving logic). Each change deserves its own commit.
- **Incremental Progress**: You added a library? Commit. You improved something? Commit.

## Governance

### Constitution Supremacy

This document is the supreme source of truth for engineering decisions. In conflicts between speed and these principles, these principles prevail.

### Amendments

Amendments require consensus and a version bump.

- **MAJOR**: Removing or redefining a core principle.
- **MINOR**: Adding a new principle or substantial section.
- **PATCH**: Clarifications and non-semantic updates.

**Version**: 1.1.0 | **Ratified**: 2025-11-27 | **Last Amended**: 2025-11-27
