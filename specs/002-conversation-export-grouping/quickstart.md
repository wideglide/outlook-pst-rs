# Quickstart: Conversation Export Grouping

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [data-model.md](data-model.md) | [contracts/cli-interface.md](contracts/cli-interface.md)  
**Updated**: 2026-03-06

## Overview

Use `--conversations` with `pst-cli export` to group related messages into `conv_#####` folders when a conversation has more than one exported message.

## Prerequisites

- Build `pst-cli`:

```bash
cargo build --release -p pst-cli
```

- Have a PST file or directory of PST files.

## Basic Conversation Grouping

```bash
./target/release/pst-cli export ./test/example-001.pst \
  --output ./out \
  --conversations
```

Expected behavior:
- Messages in multi-message conversations are placed under `./out/conv_00001/00001`, etc.
- Messages with singleton conversation keys remain at `./out/00042` style paths.
- Messages without conversation keys remain at root sequence paths.

## Conversation Grouping with Metadata

```bash
./target/release/pst-cli export ./test/example-001.pst \
  --output ./out-meta \
  --conversations \
  --metadata
```

Expected behavior:
- `metadata.txt` contains `ConversationId: ...` only when `PidTagConversationId` is present.
- Fallback-only grouped messages do not get fabricated ConversationId values.

## Combined with Existing Options

```bash
./target/release/pst-cli export ./test \
  --output ./out-full \
  --conversations \
  --metadata \
  --attachments \
  --headers \
  --csv
```

Expected behavior:
- Existing optional outputs still work.
- Conversation foldering applies only when group size > 1.
- Deterministic folder numbering follows ascending minimum sequence number per conversation.

## Quick Validation Checklist

1. Confirm at least one `conv_#####` directory exists for known multi-message conversations.
2. Confirm singleton conversation-key messages stay outside conversation folders.
3. Confirm all message sequence directories appear exactly once.
4. Confirm ConversationId appears in metadata only for messages with `PidTagConversationId`.

## Validation Snapshot (2026-03-07)

- Expected grouped layout:
  - `./out/conv_00001/00001/message.html`
  - `./out/conv_00001/00002/message.html`
- Expected singleton layout:
  - `./out/00003/message.html`
- Expected short ConversationIndex (len < 22) layout:
  - `./out/00004/message.html`
- Expected metadata behavior:
  - Message with `PidTagConversationId` includes `ConversationId: <value>`
  - Fallback-only grouped message omits `ConversationId:` line
