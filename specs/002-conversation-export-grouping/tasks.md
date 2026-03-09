# Tasks: Conversation Export Grouping

**Input**: Design documents from `/specs/002-conversation-export-grouping/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include unit, integration, and example-based validation tasks to satisfy constitution testing requirements.

**Organization**: Tasks are grouped by user story to enable independent implementation and validation of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Every task includes an exact file path

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add CLI and export data scaffolding required for conversation-aware routing.

- [X] T001 Add `--conversations` flag to export arguments in `crates/pst-cli/src/cli/mod.rs`
- [X] T002 Create conversation grouping data types and assignment helpers supporting 16-byte conversation keys in `crates/pst-cli/src/export/conversation.rs`
- [X] T003 Wire `conversation` module in `crates/pst-cli/src/export/mod.rs`
- [X] T004 Extend `MessageData` with optional conversation fields in `crates/pst-cli/src/export/exporter.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Refactor export coordination so final output path decisions are made deterministically before writing files.

**⚠️ CRITICAL**: This phase must complete before user story phases.

- [X] T005 Refactor export coordinator to stage message export records before filesystem writes in `crates/pst-cli/src/export/mod.rs`
- [X] T006 Implement deterministic `conv_#####` numbering by ascending group minimum sequence in `crates/pst-cli/src/export/conversation.rs`
- [X] T007 [P] Update exporter path composition for optional conversation subfolder in `crates/pst-cli/src/export/exporter.rs`
- [X] T008 Add coordinator guard so behavior is unchanged when `--conversations` is not set in `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: Foundation complete - user stories can now be implemented.

---

## Phase 3: User Story 1 - Group Related Messages (Priority: P1) 🎯 MVP

**Goal**: Group related messages by `PidTagConversationId` into deterministic conversation folders.

**Independent Test**: Run export with `--conversations` against PST data containing multi-message ConversationId threads and verify grouped output under one `conv_#####` directory per thread.

### Tests for User Story 1

- [X] T024 [US1] Add unit tests for ConversationId grouping and deterministic `conv_#####` numbering in `crates/pst-cli/tests/unit/conversation_grouping_test.rs`
- [X] T025 [US1] Add integration test for multi-message ConversationId export layout in `crates/pst-cli/tests/integration/conversation_grouping_test.rs`

### Implementation for User Story 1

- [X] T009 [US1] Extract `PidTagConversationId` property into the staged export record in `crates/pst-cli/src/export/mod.rs`
- [X] T010 [US1] Build ConversationId-based grouping map for exported records in `crates/pst-cli/src/export/mod.rs`
- [X] T011 [US1] Route groups with more than one message to `conv_#####/<seq>` paths in `crates/pst-cli/src/export/mod.rs`
- [X] T012 [US1] Preserve existing duplicate classification while applying conversation folder routing in `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: User Story 1 is independently functional.

---

## Phase 4: User Story 2 - Fallback Conversation Detection (Priority: P2)

**Goal**: Apply primary grouping using bytes 6-21 of `PidTagConversationIndex`, with `PidTagConversationId` as the alternate key.

**Independent Test**: Run export with `--conversations` on data where ConversationIndex is present and verify shared extracted-byte groups are exported together, with ConversationId used only when no valid index key is available.

### Tests for User Story 2

- [X] T026 [US2] Add unit tests for primary derivation from `PidTagConversationIndex` bytes 6-21 and short-index no-key behavior in `crates/pst-cli/tests/unit/conversation_key_derivation_test.rs`
- [X] T027 [US2] Add integration test for index-derived grouping and ungrouped short-index messages in `crates/pst-cli/tests/integration/conversation_fallback_test.rs`

### Implementation for User Story 2

- [X] T013 [US2] Extract raw `PidTagConversationIndex` binary values in `crates/pst-cli/src/export/mod.rs`
- [X] T014 [US2] Derive the primary key from bytes 6-21 of `PidTagConversationIndex`, using `PidTagConversationId` only when no valid index key is available, in `crates/pst-cli/src/export/mod.rs`
- [X] T015 [US2] Enforce `PidTagConversationIndex` length >= 22 bytes before fallback key derivation in `crates/pst-cli/src/export/mod.rs`
- [X] T016 [US2] Add `ConversationIndexBytes([u8; 16])` grouping key support (bytes 6-21 extraction) in `crates/pst-cli/src/export/conversation.rs`
- [X] T017 [US2] Keep messages without valid fallback key ungrouped at root sequence paths in `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: User Story 2 is independently functional.

---

## Phase 5: User Story 3 - Preserve Flat Export for Singles and Metadata Clarity (Priority: P3)

**Goal**: Keep singleton keyed messages out of conversation folders and write ConversationId to metadata only when present.

**Independent Test**: Run mixed dataset export and verify only multi-message groups use conversation folders, while metadata includes `ConversationId` only for messages containing `PidTagConversationId`.

### Tests for User Story 3

- [X] T028 [US3] Add unit tests for singleton no-folder behavior and metadata ConversationId inclusion rules in `crates/pst-cli/tests/unit/conversation_metadata_test.rs`
- [X] T029 [US3] Add integration test for mixed dataset (grouped, singleton, and unkeyed) in `crates/pst-cli/tests/integration/conversation_mixed_layout_test.rs`

### Implementation for User Story 3

- [X] T018 [US3] Enforce singleton conversation groups to remain on normal sequence paths in `crates/pst-cli/src/export/conversation.rs`
- [X] T019 [US3] Add conditional `ConversationId:` metadata output when ConversationId exists in `crates/pst-cli/src/export/metadata.rs`
- [X] T020 [US3] Populate metadata formatter input with extracted ConversationId without fallback fabrication in `crates/pst-cli/src/export/mod.rs`

**Checkpoint**: User Story 3 is independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Align feature documentation and usage guidance.

- [X] T021 [P] Document `--conversations` usage and folder layout examples in `crates/pst-cli/README.md`
- [X] T022 [P] Add conversation-grouping release notes in `crates/pst-cli/CHANGELOG.md`
- [X] T023 Run quickstart validation scenarios and record final expected outputs in `specs/002-conversation-export-grouping/quickstart.md`
- [X] T030 [P] Add example-based validation scenario for `--conversations` in `crates/pst-cli/examples/basic_export.rs`
- [X] T031 Add benchmark/regression measurement for conversation grouping overhead in `crates/pst-cli/tests/bench/conversations_bench.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies.
- **Phase 2 (Foundational)**: Depends on Phase 1 and blocks all user stories.
- **Phase 3 (US1)**: Depends on Phase 2.
- **Phase 4 (US2)**: Depends on Phase 2 and reuses shared grouping pipeline.
- **Phase 5 (US3)**: Depends on Phase 2 and uses grouping + metadata plumbing.
- **Phase 6 (Polish)**: Depends on desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Independent after foundational phase.
- **US2 (P2)**: Independent after foundational phase; extends key derivation logic.
- **US3 (P3)**: Independent after foundational phase; enforces singleton and metadata behavior.

### Within Each User Story

- Extract/derive conversation data before assignment.
- Complete story-scoped test tasks before finalizing implementation tasks.
- Assign group folder (or root fallback) before file writes.
- Render metadata after final message context is computed.

---

## Parallel Opportunities

- T007 can run in parallel with T006 after T005 establishes staging flow.
- T013 and T016 can run in parallel before integrating index-key derivation.
- T024 and T025 can run in parallel in US1 testing.
- T026 and T027 can run in parallel in US2 testing.
- T028 and T029 can run in parallel in US3 testing.
- T021 and T022 can run in parallel during polish.
- T030 can run in parallel with T021 and T022 during polish.

---

## Parallel Example: User Story 2

```bash
Task T013: Extract PidTagConversationIndex binary values in crates/pst-cli/src/export/mod.rs
Task T016: Add ConversationIndexPrefix([u8; 22]) key support in crates/pst-cli/src/export/conversation.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate US1 independently with known ConversationId threads.

### Incremental Delivery

1. Deliver US1 for primary conversation grouping.
2. Add US2 index-key grouping for broader PST compatibility.
3. Add US3 singleton and metadata behavior refinement.
4. Complete Phase 6 documentation and quickstart validation.

### Parallel Team Strategy

1. Complete setup and foundational work together.
2. Implement US2 and US3 concurrently after US1 data extraction primitives stabilize.
3. Merge all stories and finish polish tasks.
