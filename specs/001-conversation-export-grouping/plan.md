# Implementation Plan: Conversation Export Grouping

**Branch**: `001-conversation-export-grouping` | **Date**: 2026-03-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-conversation-export-grouping/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a `--conversations` export mode to `pst-cli export` that groups multi-message conversations into deterministic `conv_#####` subfolders while preserving existing message numbering and duplicate handling. Conversation membership is derived by key precedence: bytes 6-21 of `PidTagConversationIndex` first, fallback to `PidTagConversationId`, and messages without either key remain ungrouped. Technical approach uses a lightweight grouping pass during export coordination plus path composition updates in the exporter and metadata enrichment when `PidTagConversationId` is present.

## Technical Context

**Language/Version**: Rust 1.82 (edition 2021)  
**Primary Dependencies**: `outlook-pst` (workspace crate), `clap` 4.5 (CLI parsing), existing `pst-cli` export modules (`export/mod.rs`, `export/exporter.rs`, `export/metadata.rs`)  
**Storage**: Filesystem output tree under user-provided `--output` directory  
**Testing**: `cargo test -p pst-cli` with unit tests and integration tests under `crates/pst-cli/tests/`  
**Target Platform**: Cross-platform CLI (macOS/Linux/Windows)  
**Project Type**: Single workspace CLI crate (`crates/pst-cli`)  
**Performance Goals**: Keep export throughput within existing expectations while adding only O(n) grouping overhead for n exported messages  
**Constraints**: Preserve deterministic output, preserve existing duplicate behavior, avoid unsafe code, maintain one output location per exported message  
**Scale/Scope**: Conversation grouping for all messages processed in one export run, including directory batch mode across multiple PST inputs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Gate Review

### Code Quality (NON-NEGOTIABLE)
PASS:
- Feature is additive and localized to existing export/CLI modules.
- Safe Rust only; no `unsafe` needed.
- Behavior is encoded as deterministic rules in spec and contract.

### Testing Standards (NON-NEGOTIABLE)
PASS:
- Plan includes unit coverage for key derivation and folder numbering.
- Plan includes integration scenarios for mixed conversation/non-conversation exports and singleton handling.

### User Experience Consistency
PASS:
- Adds one explicit opt-in flag (`--conversations`) consistent with current flag style.
- Existing output layout remains unchanged unless flag is enabled.

### Performance Requirements
PASS:
- Grouping strategy is linear in message count and avoids expensive per-message rescans.
- No changes to core PST parsing path; work occurs after message data extraction.

### Post-Design Gate Re-Check

PASS:
- `research.md` resolves key strategy, folder numbering, and duplicate interaction decisions.
- `data-model.md` defines conversation entities and validation rules.
- `contracts/cli-interface.md` defines external CLI behavior and output structure changes.
- `quickstart.md` provides executable usage and validation flow.

**Constitution Compliance**: All gates PASS. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/001-conversation-export-grouping/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-interface.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/pst-cli/
├── src/
│   ├── cli/
│   │   └── mod.rs                  # Add --conversations flag
│   ├── export/
│   │   ├── mod.rs                  # Extract conversation properties, group assignment logic
│   │   ├── exporter.rs             # Conversation-aware output path composition
│   │   └── metadata.rs             # ConversationId line in metadata.txt
│   └── main.rs                     # Existing command dispatch (no interface shape change)
└── tests/
    ├── integration/                # End-to-end export layout assertions
    └── unit/                       # Conversation-key and numbering logic tests
```

**Structure Decision**: Keep the existing single-crate CLI structure and implement conversation grouping inside current export pipeline modules, minimizing architectural change while preserving deterministic behavior and compatibility.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations identified.
