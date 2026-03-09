# Feature Specification: Conversation Export Grouping

**Feature Branch**: `[001-conversation-export-grouping]`  
**Created**: 2026-03-06  
**Status**: Draft  
**Input**: User description: "Add a feature to export functionality where `--conversations` identifies conversations using bytes 6-21 of `PidTagConversationIndex` (16 bytes), or falls back to `PidTagConversationId` (16 bytes); messages in multi-message conversations export together under `conv_00001` style subfolders; single-message conversations stay at root; include ConversationId in `metadata.txt` when present."

## Clarifications

### Session 2026-03-06

- Q: How should `conv_00001`, `conv_00002`, ... numbering be assigned to conversation groups? -> A: Number conversation folders by ascending minimum message sequence number in each group.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Group Related Messages (Priority: P1)

As a user exporting messages, I want related messages from the same conversation to be grouped together so I can review entire conversation threads without manually finding each message.

**Why this priority**: Grouping multi-message conversations is the core user value of the feature and drives the `--conversations` behavior.

**Independent Test**: Run an export with `--conversations` on data containing at least one conversation with multiple messages, then verify all messages in that conversation appear under a common conversation subfolder.

**Acceptance Scenarios**:

1. **Given** export input where several messages share the same `PidTagConversationId`, **When** export is run with `--conversations`, **Then** those messages are exported into the same conversation subfolder.
2. **Given** export input with multiple distinct conversations, **When** export is run with `--conversations`, **Then** each conversation with more than one message is exported into its own subfolder and no message appears in multiple conversation subfolders.

---

### User Story 2 - Fallback Conversation Detection (Priority: P2)

As a user exporting from varied PST data, I want conversation grouping to still work when `PidTagConversationId` is absent so that legacy or incomplete metadata does not prevent useful grouping.

**Why this priority**: Fallback detection broadens compatibility and preserves grouping value across more real-world message sets.

**Independent Test**: Run an export with `--conversations` where `PidTagConversationId` is missing but `PidTagConversationIndex` exists with bytes 6-21 identical across multiple messages, then verify those messages are grouped together.

**Acceptance Scenarios**:

1. **Given** messages without `PidTagConversationId` but with `PidTagConversationIndex` at least 22 bytes long, **When** export is run with `--conversations`, **Then** grouping uses bytes 6-21 of `PidTagConversationIndex` as the conversation key (the first 6 bytes are reserved/timestamp).
2. **Given** a message lacking both `PidTagConversationId` and `PidTagConversationIndex`, **When** export is run with `--conversations`, **Then** that message is exported without conversation grouping.

---

### User Story 3 - Preserve Flat Export for Singles and Metadata Clarity (Priority: P3)

As a user, I want single-message conversations to remain in the normal export layout and to see conversation identifiers in metadata when available so that output stays clean while still being traceable.

**Why this priority**: Preventing unnecessary subfolders reduces clutter, while metadata enrichment improves auditability and troubleshooting.

**Independent Test**: Run an export with `--conversations` on data containing both single-message and multi-message conversation keys, then verify only multi-message groups get subfolders and metadata includes ConversationId where present.

**Acceptance Scenarios**:

1. **Given** a conversation key that appears on exactly one message, **When** export is run with `--conversations`, **Then** that message remains in the normal export location and is not moved into a conversation subfolder.
2. **Given** a message with `PidTagConversationId`, **When** metadata is written, **Then** `metadata.txt` includes the ConversationId value for that message.

### Edge Cases

- Two or more messages may share a conversation key derived from bytes 6-21 of `PidTagConversationIndex` while lacking `PidTagConversationId`; these messages are treated as one conversation for grouping.
- A conversation key may exist on only one message in the export scope; that message stays at root and no empty conversation folder is created.
- Export scope may include only a subset of a larger conversation; grouping decisions are based only on messages included in the current export run.
- Messages without any conversation key remain exportable and are not blocked by conversation grouping logic.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support a `--conversations` export mode that enables conversation-aware grouping behavior.
- **FR-002**: When `--conversations` is enabled and `PidTagConversationIndex` exists and is at least 22 bytes, the system MUST use bytes 6-21 of `PidTagConversationIndex` (the 16-byte conversation identity, after the 6-byte reserved/timestamp prefix) as the primary conversation key.
- **FR-003**: When `--conversations` is enabled and no valid `PidTagConversationIndex` key is available, if `PidTagConversationId` is present, the system MUST use `PidTagConversationId` as the alternate conversation key; otherwise, the system MUST use no conversation key.
- **FR-004**: When no conversation key is available, the system MUST export the message without assigning it to a conversation group.
- **FR-005**: System MUST group messages into a conversation subfolder only when more than one exported message shares the same conversation key within the current export run.
- **FR-006**: System MUST NOT create a conversation subfolder for a conversation key that has exactly one exported message.
- **FR-007**: Conversation subfolders MUST follow a deterministic sequential naming format equivalent to `conv_00001`, `conv_00002`, and so on, within a single export run.
- **FR-012**: Conversation folder numbering MUST be assigned by ascending minimum message sequence number across conversation groups in that export run.
- **FR-008**: Messages placed in a conversation subfolder MUST preserve the existing per-message naming convention under that subfolder.
- **FR-009**: For messages where `PidTagConversationId` is present, system MUST include the ConversationId value in that message's `metadata.txt`.
- **FR-010**: For messages where `PidTagConversationId` is absent, system MUST NOT write a fabricated ConversationId value to `metadata.txt`.
- **FR-011**: Export output MUST contain each exported message exactly once, either in a conversation subfolder or in the normal root-level export structure.

### Key Entities *(include if feature involves data)*

- **Exported Message**: A single message selected for export, including sequence number, output path, and metadata file.
- **Conversation Key**: A grouping identifier derived from bytes 6-21 of `PidTagConversationIndex` (16 bytes after the reserved/timestamp prefix) when available, otherwise from `PidTagConversationId` (16 bytes).
- **Conversation Group**: A collection of two or more exported messages sharing the same Conversation Key within one export run.
- **Conversation Folder**: The output subfolder representing a Conversation Group, named using the sequential `conv_#####` pattern.

### Assumptions

- Conversation grouping is applied only when `--conversations` is explicitly provided.
- Grouping evaluation uses only messages included in the current export scope, not messages outside that run.
- Existing export behavior and format remain unchanged when `--conversations` is not provided.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In validation datasets containing at least 20 known multi-message conversations, 100% of messages belonging to those conversations are exported into the correct shared conversation subfolder.
- **SC-002**: In validation datasets where at least 20 conversations rely on the extracted `ConversationIndex` key, at least 99% of messages are grouped according to the expected grouping.
- **SC-003**: In validation datasets containing at least 50 single-message conversation keys, 100% of those messages remain outside conversation subfolders.
- **SC-004**: In validation datasets where `PidTagConversationId` is present, 100% of corresponding `metadata.txt` files include a ConversationId entry.
- **SC-005**: In sampled export runs, 100% of exported messages appear exactly once in the output structure.
