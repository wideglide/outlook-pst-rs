# Research: Conversation Export Grouping

**Phase**: 0 - Research & Design Decisions  
**Date**: 2026-03-06  
**Feature**: [spec.md](spec.md) | [plan.md](plan.md)

## Overview

This document captures design choices for adding `--conversations` to `pst-cli export` while preserving deterministic output and current export behavior.

## Conversation Key Derivation

### Decision
Use a two-level key derivation strategy:
1. Use bytes 6-21 of `PidTagConversationIndex` when present.
2. Otherwise, use `PidTagConversationId` when present.
3. If neither property exists, treat the message as not conversation-grouped.

### Rationale
- Matches feature requirements exactly.
- Prefers the extracted conversation-index identity when available.
- Provides robust fallback for messages missing ConversationId.
- Avoids fabricated keys for messages with no conversation properties.

### Alternatives considered
- Use only `PidTagConversationId`: rejected because the preferred extracted conversation-index key would be ignored.
- Use full `PidTagConversationIndex`: rejected because requirement specifies bytes 6-21 only.
- Hash message content as fallback: rejected because this changes semantic meaning from conversation grouping to duplicate-style grouping.

## Folder Numbering Strategy

### Decision
Assign `conv_#####` folder numbers by ascending minimum exported message sequence number for each conversation group.

### Rationale
- Aligned with approved clarification in spec.
- Deterministic and independent from hash map iteration order.
- Stable across runs when input ordering is stable.

### Alternatives considered
- Lexical sort by conversation key bytes: deterministic but less intuitive for users.
- First-seen traversal order: deterministic only if traversal remains unchanged; less explicit than sequence-based rule.

## Group Eligibility Rule

### Decision
Create conversation folders only for groups with more than one exported message in the current run. Keep singleton keyed messages in the normal root sequence path.

### Rationale
- Matches functional requirements FR-005 and FR-006.
- Prevents unnecessary folder depth and preserves current output readability.

### Alternatives considered
- Always create a folder for any keyed message: rejected because it violates singleton requirement.
- Group by conversation across all available PST files on disk: rejected because export scope is only current run input.

## Duplicate Handling Compatibility

### Decision
Preserve existing duplicate routing behavior and apply conversation subfoldering inside each existing root context:
- Non-duplicates under `<output>/...`
- Duplicates under `<output>/duplicates/...`

Conversation grouping logic is applied to exported messages before final path composition, without changing duplicate detection semantics.

### Rationale
- Keeps current duplicate model intact.
- Avoids broad behavioral changes not requested by feature.
- Supports deterministic single placement per message (FR-011).

### Alternatives considered
- Ignore conversation grouping for duplicates: rejected because it creates inconsistent behavior for same flag.
- Merge duplicates into non-duplicate conversation folders: rejected because it breaks current duplicate separation contract.

## Metadata Behavior

### Decision
Write `ConversationId: <value>` in `metadata.txt` only when `PidTagConversationId` exists. Do not emit derived fallback bytes as ConversationId.

### Rationale
- Matches requirement to write ConversationId only when present.
- Avoids mislabeling fallback key as canonical conversation ID.

### Alternatives considered
- Emit fallback key as ConversationId when ID missing: rejected as semantically inaccurate.
- Emit empty ConversationId line always: rejected as unnecessary noise.

## Implementation Shape

### Decision
Implement grouping in export coordination with minimal structural change:
- Add `--conversations` flag in `cli/mod.rs`.
- Extend extracted message model with optional conversation fields.
- Build conversation grouping map keyed by derived key.
- Compute eligible multi-message groups and deterministic `conv_#####` mapping.
- Pass optional conversation folder assignment to exporter path logic.
- Update metadata formatter for optional ConversationId line.

### Rationale
- Uses existing extension points and keeps code localized.
- Avoids introducing a new subsystem for a small feature.
- Makes test boundaries clear (key derivation, numbering, path composition).

### Alternatives considered
- Introduce a dedicated conversation engine module first: rejected as over-design for current scope.

## Resolved Clarifications

All technical unknowns in this feature are resolved. No `NEEDS CLARIFICATION` items remain.
