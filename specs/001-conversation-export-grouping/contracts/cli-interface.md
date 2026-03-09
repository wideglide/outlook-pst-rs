# CLI Interface Contract: Conversation Export Grouping

**Feature**: [../spec.md](../spec.md) | [../plan.md](../plan.md) | [../data-model.md](../data-model.md)  
**Phase**: 1 - Design  
**Date**: 2026-03-06

## Overview

This contract defines the external CLI behavior added by the conversation-grouping feature for `pst-cli export`.

## Command Surface Change

### Command
`pst-cli export`

### New Optional Flag
`--conversations`

### Semantics
When `--conversations` is enabled:
- Conversation key derivation precedence:
1. If `PidTagConversationIndex` exists and is at least 22 bytes, use bytes 6-21.
2. Else, use `PidTagConversationId` when present.
3. Else, use no conversation key.
- Only groups with more than one exported message are placed in conversation folders.
- Conversation folders are named `conv_00001`, `conv_00002`, ...
- Folder numbering order is ascending minimum message sequence number in each group.
- Messages with a conversation key but group size 1 remain at normal root sequence path.
- Messages without conversation key remain at normal root sequence path.

When `--conversations` is not enabled:
- Existing export layout and behavior remain unchanged.

## Output Layout Contract

### Baseline (existing)
- Message directories: `<output>/<seq>/`
- Duplicate directories: `<output>/duplicates/<seq>/`

### With `--conversations`
- Grouped messages (group size > 1):
  - Main root: `<output>/conv_#####/<seq>/`
  - Duplicates root (if duplicate behavior applies): `<output>/duplicates/conv_#####/<seq>/`
- Singleton keyed messages:
  - Main root: `<output>/<seq>/`
  - Duplicates root: `<output>/duplicates/<seq>/`
- Unkeyed messages:
  - Same as singleton keyed messages.

## Metadata Contract

For message `metadata.txt` output:
- Include `ConversationId: <value>` only when `PidTagConversationId` exists on that message.
- Do not synthesize `ConversationId` from fallback key bytes.

## Determinism Contract

For a fixed input set and unchanged traversal order:
- Sequence numbering remains deterministic.
- Conversation folder assignment remains deterministic by minimum sequence rule.
- Each exported message appears exactly once.

## Validation Scenarios

1. Two messages share `PidTagConversationId`.
- Expect both under one `conv_#####` folder.

2. Two messages lack ConversationId but share bytes 6-21 of ConversationIndex.
- Expect both under one `conv_#####` folder.

3. Single message has conversation key.
- Expect no `conv_#####` folder for that message.

4. Message lacks both conversation properties.
- Expect no conversation grouping.

5. Message has `PidTagConversationId` and metadata export enabled.
- Expect `ConversationId:` line in `metadata.txt`.

6. Message lacks `PidTagConversationId` but has fallback key.
- Expect no `ConversationId:` line in `metadata.txt`.

7. Message has `PidTagConversationIndex` shorter than 22 bytes and no `PidTagConversationId`.
- Expect no conversation grouping for that message.
