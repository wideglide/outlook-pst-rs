<!--
Sync Impact Report

- Version change: 0.1.0 -> 0.2.0
- Modified principles:
  - Portability & Correctness -> Code Quality (expanded)
  - Library-First Crate Boundaries -> Testing Standards (new focus)
  - Documentation & Examples -> User Experience Consistency (new focus)
  - Test-First (NON-NEGOTIABLE) -> Testing Standards (refined)
  - Observability & Performance -> Performance Requirements (refined)
- Added sections: explicit Code Quality and UX guidance
- Removed sections: none
- Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ reviewed (Constitution Check present)
  - .specify/templates/spec-template.md: ✅ reviewed
  - .specify/templates/tasks-template.md: ✅ reviewed
  - .specify/templates/commands/: ⚠ verify existence and agent-specific names
- Follow-up TODOs:
  - TODO(RATIFICATION_DATE): set original ratification date upon formal adoption
  - Verify command templates under .specify/templates/commands/ and update any agent-specific references
-->

# Outlook PST Store Provider (Rust) Constitution

## Core Principles

### Code Quality (NON-NEGOTIABLE)
All code MUST meet high quality standards: clear, idiomatic Rust, minimal unsafe code, and
strong type safety. Code MUST be self-documenting where possible and include concise, focused
module and item documentation. Breaking the following rules requires an explicit design note in
the PR explaining why the deviation is necessary and how risks are mitigated.

Rationale: high code quality reduces security and maintenance cost for a complex binary-format
implementation and enables safer automated assistance and audits.

Key requirements:
- Prefer safe Rust; `unsafe` blocks MUST be isolated, reviewed, and accompanied by safety
  comments explaining invariants.
- Public APIs MUST be minimal and stable; avoid unnecessary pub exposure.
- Follow repository `rust-guidelines.txt` and crate-level linting/formatting rules.

### Testing Standards (NON-NEGOTIABLE)
Testing is mandatory and tiered: unit tests for behavior, integration tests for cross-crate
contracts, and example-based tests for expected user scenarios. Tests MUST be written before
implementation when feasible. Every bug fix MUST include a regression test.

Rationale: PST parsing is fragile; comprehensive tests prevent regressions and provide a safety
net for refactors and performance changes.

Key requirements:
- Unit tests: cover public API invariants and edge cases.
- Integration tests: exercise crate interactions and on-disk formats using representative fixtures.
- Example-based tests: runnable examples in `examples/` that are executed in CI.
- Benchmarks: hot paths MUST have benchmarks; significant performance work MUST include
  before/after benchmark results in the PR.

### User Experience Consistency
Command-line tools, examples, and library APIs MUST present consistent UX patterns: predictable
error reporting, stable flag names, and human-readable defaults. Documentation and examples MUST
demonstrate the recommended developer and user flows.

Rationale: consistent UX reduces cognitive load for integrators and helps tests and examples be
more meaningful across crates.

Key requirements:
- CLI tools and examples MUST use a shared style for output and error formatting.
- Errors MUST be actionable: include context, expected input, and suggested fixes where
  appropriate.
- Maintain a small set of canonical examples that serve as the primary user-facing guidance.

### Performance Requirements
Performance goals MUST be explicit for identified hot paths. Work that changes performance
characteristics MUST be accompanied by measurable benchmarks and profiling artifacts.

Rationale: PST operations can be CPU- and IO-bound; measurable goals prevent regressions and
ensure practical performance for real workloads.

Key requirements:
- Identify hot paths and add benchmarks (e.g., using `criterion`) in the relevant crate.
- Include profiling notes and flamegraphs when proposing optimizations.
- Avoid premature optimization: profile first, then optimize the measured bottleneck.

## Constraints & Security Requirements

- Licensing: Contributors MUST complete required CLAs and follow repository licensing.
- Security: Parse untrusted inputs defensively. Validate sizes, offsets, and counts before
  allocating. Prefer bounds-checked operations and fail early on malformed input.
- FFI: FFI surfaces MUST use well-documented representations and avoid moving owned standard
  library containers (e.g., `String`, `Vec`) across ABI boundaries without clear ownership rules.

## Development Workflow & How Principles Guide Decisions

- Design notes: Non-trivial technical decisions MUST include a short design note describing how
  the choice aligns with the constitution (code quality, testing, UX, performance) and listing
  trade-offs.
- PR requirements: Every PR MUST include a short checklist addressing:
  - Code quality: linting, docs, and `unsafe` justification (if any)
  - Tests: new/updated tests and the rationale for coverage decisions
  - UX: any user-facing changes and compatibility notes
  - Performance: benchmark/impact notes if performance is affected
- Review guidance: Reviewers MUST verify the PR checklist and ensure tests run in CI. For
  performance-impacting changes, at least one approving maintainer MUST verify benchmarks.

## Governance

Amendments and governance policy:

1. Propose an amendment via an issue or spec that references this constitution and includes a
   migration plan for affected artifacts.
2. The proposal MUST include an explicit version-bump recommendation (MAJOR/MINOR/PATCH) with
   rationale. Add semantic versioning guidance: MINOR for new principles/expansions, PATCH for
   clarifications, MAJOR for breaking governance changes.
3. Approval: at least two maintainers OR one maintainer + one domain expert must approve.
4. After approval, update the constitution file and set `Last Amended` to the amendment date.

Compliance expectations:

- Policy checks: CI or review scripts SHOULD include a lightweight constitution checklist for PRs
  touching critical areas (parsing, FFI, core libraries).
- Annual review: Maintainers SHOULD review the constitution annually and propose updates as
  required.

**Version**: 0.2.0 | **Ratified**: TODO(RATIFICATION_DATE): unknown — set upon formal adoption | **Last Amended**: 2026-02-05
