# Data Model: Conversation Export Grouping

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [research.md](research.md)  
**Phase**: 1 - Design  
**Date**: 2026-03-06

## Overview

This model adds conversation-aware grouping metadata to the existing export pipeline while preserving current message sequence numbering and output contracts.

## Entities

### ExportMessageRecord

Represents one message selected for export after existing draft and duplicate decisions.

**Fields**:
- `sequence_number: u32` - Existing export sequence number (`00001`, `00002`, ...).
- `is_duplicate: bool` - Existing duplicate classification.
- `base_output_root: enum { Main, Duplicates }` - Existing output root context.
- `conversation_id: Option<Vec<u8>>` - Raw `PidTagConversationId` value when present (16 bytes).
- `conversation_index: Option<Vec<u8>>` - Full `PidTagConversationIndex` bytes when available, providing fallback key extraction source.
- `conversation_key: Option<ConversationKey>` - Derived key used for grouping.
- `conversation_folder: Option<String>` - Assigned conversation folder (for multi-message groups only).

**Validation rules**:
- `sequence_number` is unique within one run.
- `conversation_key` is present only when either ConversationId or fallback prefix exists.
- `conversation_folder` is present only when group size > 1.

### ConversationKey

Canonical grouping key used within one export run. Both key variants are 16-byte binary identifiers.

**Variants**:
- `ConversationId([u8; 16])` - Derived from `PidTagConversationId`.
- `ConversationIndexBytes([u8; 16])` - Derived from bytes 6-21 of `PidTagConversationIndex` (after 6-byte reserved/timestamp prefix).

**Derivation precedence**:
1. Bytes 6-21 of `PidTagConversationIndex` (16 bytes, requires >= 22 total bytes)
2. `PidTagConversationId` (16 bytes)
3. No key

**Validation rules**:
- `ConversationId` key must be exactly 16 bytes.
- `ConversationIndexBytes` extraction requires full `PidTagConversationIndex` >= 22 bytes, extracting only bytes 6-21.
- Keys are compared by exact byte equality.

### ConversationGroup

Represents exported messages sharing the same `ConversationKey`.

**Fields**:
- `key: ConversationKey`
- `members: Vec<u32>` - Sequence numbers of messages in the run with this key.
- `group_size: usize`
- `min_sequence: u32`
- `folder_name: Option<String>` - `Some("conv_#####")` when `group_size > 1`, otherwise `None`.

**Validation rules**:
- `group_size == members.len()`
- `min_sequence` equals minimum of `members`.
- `folder_name` exists only when `group_size > 1`.

### ConversationFolderAssignment

Deterministic mapping from eligible groups to output folder names.

**Fields**:
- `group_key: ConversationKey`
- `folder_name: String` - `conv_00001`, `conv_00002`, ...
- `order_rank: u32` - Rank by ascending `min_sequence`.

**Validation rules**:
- Folder names are contiguous with no gaps for eligible groups.
- Assignment order strictly follows ascending `min_sequence`.

## Relationships

- One `ExportMessageRecord` has zero or one `ConversationKey`.
- One `ConversationGroup` has one `ConversationKey` and one or more `ExportMessageRecord` members.
- One `ConversationFolderAssignment` maps one eligible `ConversationGroup` (`group_size > 1`) to one folder name.

## State Transitions

1. `Extracted`: Message fields loaded from PST properties.
2. `Keyed`: Conversation key derived (or absent).
3. `Grouped`: Message associated with conversation group bucket.
4. `Assigned`: Multi-message groups receive deterministic `conv_#####` folder.
5. `Exported`: Message written once to final path (with or without conversation subfolder).

## Invariants

- Each exported message is written exactly once.
- Single-message conversation groups do not receive conversation subfolders.
- Messages without conversation key always use normal sequence-based directory layout.
- Metadata includes `ConversationId` only when `conversation_id` field is present.
- The first 6 bytes of `PidTagConversationIndex` are reserved and not used for grouping.
