# Tasks: HTML Search and Inline Export

**Input**: Design documents from `/specs/003-html-search-inline-export/`  
**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required), [research.md](research.md), [data-model.md](data-model.md), [contracts/cli-interface.md](contracts/cli-interface.md), [quickstart.md](quickstart.md)

**Tests**: Include regression-focused unit and integration tests because the constitution and plan require coverage for behavior changes in filtering and export.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: User story label for traceability (`US1`, `US2`, `US3`)
- **File paths**: Always included in task description

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare dependencies and test entry points needed by the feature

- [X] T001 Add the parse-based HTML dependency to `crates/pst-cli/Cargo.toml` for visible-text extraction and inline HTML rewriting
- [X] T002 [P] Register any new regression test modules needed for this feature in `crates/pst-cli/tests/unit.rs` and `crates/pst-cli/tests/integration.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core export-pipeline infrastructure that MUST be complete before any user story work can be finished

**⚠️ CRITICAL**: No user story can be completed until this phase is done

- [X] T003 [P] Extend attachment and export data structures with inline-resolution metadata and attachment-plan types in `crates/pst-cli/src/export/exporter.rs`
- [X] T004 [P] Capture `PidTagAttachContentId` and `PidTagAttachContentLocation` during attachment extraction in `crates/pst-cli/src/export/mod.rs`
- [X] T005 Create shared attachment filename sanitization, collision handling, and export-plan generation in `crates/pst-cli/src/export/exporter.rs`
- [X] T006 [P] Add shared HTML utilities for visible-text extraction, inline-reference normalization, and parse-based rewriting in `crates/pst-cli/src/export/html.rs`
- [X] T007 Wire the shared attachment plan and HTML-processing hooks into the staged export flow in `crates/pst-cli/src/export/mod.rs` and `crates/pst-cli/src/export/exporter.rs`

**Checkpoint**: Attachment metadata, filename planning, and HTML helper infrastructure are ready for story-level work.

---

## Phase 3: User Story 1 - Search Visible HTML Content Only (Priority: P1) 🎯 MVP

**Goal**: Make keyword search on HTML bodies use visible text only, excluding tags, scripts, styles, and comments.

**Independent Test**: Run export with `--keywords` against HTML messages where the keyword appears once in visible text and once only in markup/script/style/comment content; only the visible-text message should report the keyword hit.

### Tests for User Story 1

- [X] T008 [P] [US1] Add unit tests for visible-text extraction and malformed-HTML handling in `crates/pst-cli/tests/unit/html_test.rs`
- [X] T009 [P] [US1] Add unit tests for HTML-aware keyword matching regressions in `crates/pst-cli/tests/unit/keyword_test.rs`
- [X] T010 [P] [US1] Add integration coverage for HTML keyword filtering in `crates/pst-cli/tests/integration/filtering_test.rs`

### Implementation for User Story 1

- [X] T011 [US1] Implement visible-text extraction that excludes tags, comments, `<script>`, and `<style>` content in `crates/pst-cli/src/export/html.rs`
- [X] T012 [US1] Update HTML body keyword matching to use extracted visible text while preserving subject and plain-text behavior in `crates/pst-cli/src/filter/keyword.rs`
- [X] T013 [US1] Integrate HTML-aware body selection into export filtering in `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: User Story 1 is independently functional and testable through `--keywords` without inline attachment work.

---

## Phase 4: User Story 2 - Resolve Inline References in Exported HTML (Priority: P2)

**Goal**: Rewrite resolvable `cid:` and content-location references in `message.html` to the exact local attachment files exported for the same message.

**Independent Test**: Export a message with inline attachments and `--attachments`; opening `message.html` from disk should load matching inline resources from the message directory.

### Tests for User Story 2

- [X] T014 [P] [US2] Add unit tests for attachment metadata normalization and filename planning in `crates/pst-cli/tests/unit/attachment_test.rs`
- [X] T015 [P] [US2] Add unit tests for `cid:` and content-location rewrite behavior in `crates/pst-cli/tests/unit/html_test.rs`
- [X] T016 [P] [US2] Add integration coverage for inline attachment rewriting in `crates/pst-cli/tests/integration/export_test.rs`

### Implementation for User Story 2

- [X] T017 [US2] Extend extracted attachments with content ID and content-location metadata in `crates/pst-cli/src/export/mod.rs` and `crates/pst-cli/src/export/exporter.rs`
- [X] T018 [US2] Implement deterministic attachment export planning with final relative filenames in `crates/pst-cli/src/export/exporter.rs`
- [X] T019 [US2] Implement parse-based rewriting of resolvable `src` and `href` inline references in `crates/pst-cli/src/export/html.rs`
- [X] T020 [US2] Apply the shared attachment export plan when writing `message.html` and attachment files in `crates/pst-cli/src/export/exporter.rs`

**Checkpoint**: User Story 2 is independently functional and testable with exported inline resources resolving to local files.

---

## Phase 5: User Story 3 - Preserve Unrelated and Unresolvable Links (Priority: P3)

**Goal**: Rewrite only valid same-message inline attachment references and leave external or unresolved references untouched.

**Independent Test**: Export a message containing a mix of external URLs, unresolved inline references, and resolvable inline references; only the resolvable references should change.

### Tests for User Story 3

- [X] T021 [P] [US3] Add unit tests for preserving external URLs, anchor links, and unmatched references in `crates/pst-cli/tests/unit/html_test.rs`
- [X] T022 [P] [US3] Add unit tests for disabled-attachments and unmatched-attachment cases in `crates/pst-cli/tests/unit/attachment_test.rs`
- [X] T023 [P] [US3] Add integration coverage for mixed resolvable, ambiguous, and unresolvable references in `crates/pst-cli/tests/integration/export_test.rs`

### Implementation for User Story 3

- [X] T024 [US3] Implement reference classification and normalization rules that skip external and non-inline URLs in `crates/pst-cli/src/export/html.rs`
- [X] T025 [US3] Treat multiple candidate attachment matches as unresolved and preserve the original HTML reference in `crates/pst-cli/src/export/html.rs` and `crates/pst-cli/src/export/exporter.rs`
- [X] T026 [US3] Preserve original HTML references when attachments are disabled or no same-message match exists in `crates/pst-cli/src/export/exporter.rs`
- [X] T027 [US3] Ensure mixed rewrite outcomes remain deterministic and leave message.html generation successful in `crates/pst-cli/src/export/exporter.rs` and `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: All three user stories are independently functional and mixed-reference export behavior is stable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish benchmarks, examples, documentation, validation, and cross-story cleanup

- [X] T028 [P] Add focused benchmarks for HTML visible-text extraction and inline rewrite hot paths in `crates/pst-cli/tests/bench/html_export_bench.rs`
- [X] T029 [P] Capture before/after benchmark results and profiling notes for the HTML filtering and export changes in `specs/003-html-search-inline-export/quickstart.md`
- [X] T030 [P] Create a canonical runnable example covering HTML keyword filtering and inline attachment export in `crates/pst-cli/examples/html_search_inline_export.rs`
- [X] T031 [P] Document the example workflow and expected outputs in `crates/pst-cli/README.md` and `specs/003-html-search-inline-export/quickstart.md`
- [X] T032 [P] Add concise module and API documentation for new HTML rewrite and attachment-plan helpers in `crates/pst-cli/src/export/html.rs` and `crates/pst-cli/src/export/exporter.rs`
- [X] T033 Ensure the new example is exercised by the repository's normal validation flow and referenced in final verification notes in `specs/003-html-search-inline-export/quickstart.md`
- [X] T034 Run the targeted validation commands from `specs/003-html-search-inline-export/quickstart.md` and the full relevant `cargo test -p pst-cli` suite

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; can start immediately
- **Foundational (Phase 2)**: Depends on Setup and blocks all story completion
- **User Stories (Phases 3-5)**: Depend on Foundational completion
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Phase 2 and has no dependency on other stories
- **User Story 2 (P2)**: Starts after Phase 2 and depends only on shared attachment-plan and HTML-rewrite infrastructure, not on US1 completion
- **User Story 3 (P3)**: Starts after Phase 2 and builds on the rewrite path from US2 while remaining independently testable

### Within Each User Story

- Tests should be written before or alongside implementation and fail before the implementation is considered complete
- Shared helpers should be completed before export-path wiring that consumes them
- Story-level integration should follow the unit-tested helpers for that story

### Parallel Opportunities

- `T002`, `T003`, `T004`, and `T006` can run in parallel after dependency setup starts
- For **US1**, `T008`, `T009`, and `T010` can run in parallel before `T011`-`T013`
- For **US2**, `T014`, `T015`, and `T016` can run in parallel before `T017`-`T020`
- For **US3**, `T021`, `T022`, and `T023` can run in parallel before `T024`-`T027`

---

## Parallel Example: User Story 1

```bash
# Launch US1 regression tests together:
Task: "Add unit tests for visible-text extraction and malformed-HTML handling in crates/pst-cli/tests/unit/html_test.rs"
Task: "Add unit tests for HTML-aware keyword matching regressions in crates/pst-cli/tests/unit/keyword_test.rs"
Task: "Add integration coverage for HTML keyword filtering in crates/pst-cli/tests/integration/filtering_test.rs"
```

---

## Parallel Example: User Story 2

```bash
# Launch US2 test coverage together:
Task: "Add unit tests for attachment metadata normalization and filename planning in crates/pst-cli/tests/unit/attachment_test.rs"
Task: "Add unit tests for cid and content-location rewrite behavior in crates/pst-cli/tests/unit/html_test.rs"
Task: "Add integration coverage for inline attachment rewriting in crates/pst-cli/tests/integration/export_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational infrastructure
3. Complete Phase 3: User Story 1
4. Validate keyword filtering independently with the HTML visible-text scenarios from `spec.md`

### Incremental Delivery

1. Deliver US1 to remove HTML keyword false positives
2. Deliver US2 to make inline resources resolve locally in exported HTML
3. Deliver US3 to harden rewrite behavior and preserve unrelated links
4. Finish Phase 6 documentation and validation

### Suggested MVP Scope

Implement through **Phase 3 / User Story 1** first. That is the smallest independently valuable slice and directly fixes the search correctness problem.

---

## Notes

- Every task follows the required checklist format with ID, optional `[P]`, optional story label, and concrete file paths
- The task list assumes the active feature directory is `specs/003-html-search-inline-export/`
- `crates/pst/src/messaging/attachment.rs` is intentionally excluded from the default path unless direct property access in `crates/pst-cli/src/export/mod.rs` proves insufficient during implementation