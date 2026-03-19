# Implementation Plan: HTML Search and Inline Export

**Branch**: `003-html-search-inline-export` | **Date**: 2026-03-09 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-html-search-inline-export/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Improve `pst-cli export` in two linked areas without changing the command surface: keyword filtering must evaluate HTML bodies using visible text only, and exported `message.html` files must rewrite resolvable inline `cid:` and content-location references to the attachment files written for the same message. The plan keeps the implementation localized to the existing `pst-cli` export/filter pipeline by introducing a single parse-based HTML processing dependency, extending attachment metadata captured during message extraction, and centralizing attachment filename planning so HTML rewriting and disk output use the same resolved paths.

## Technical Context

**Language/Version**: Rust 1.82 (edition 2021)  
**Primary Dependencies**: `outlook-pst` workspace crate, `compressed-rtf`, `clap` 4.5, `chrono`, `encoding_rs`, `html-escape`, planned parse-based HTML processor (`lol_html`)  
**Storage**: Filesystem export tree under the user-provided `--output` directory  
**Testing**: `cargo test -p pst-cli` with unit tests under `crates/pst-cli/tests/unit/` and integration tests under `crates/pst-cli/tests/integration/`  
**Target Platform**: Cross-platform CLI on macOS, Linux, and Windows
**Project Type**: Rust workspace with a CLI crate (`crates/pst-cli`) backed by a PST parsing library crate (`crates/pst`)  
**Performance Goals**: Keep keyword filtering and HTML export processing effectively single-pass per message, with no additional PST reads and no extra attachment filename resolution pass after HTML generation  
**Constraints**: Preserve current behavior for non-HTML bodies, preserve current output layout unless an inline reference can be resolved, avoid regex-based HTML parsing, keep output deterministic, stay in safe Rust  
**Scale/Scope**: All exported messages in a single run, including directory inputs spanning multiple PST files and messages with multiple attachments or repeated inline references

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Gate Review

### Code Quality (NON-NEGOTIABLE)
PASS:
- Change stays inside existing export/filter seams in `crates/pst-cli` and uses existing PST property extraction patterns.
- Safe Rust throughout (no FFI, no unsafe required for CLI and format conversion)
- Plan favors one parse-based HTML dependency over ad hoc string scanning.
- Follows `rust-guidelines.txt`: comprehensive docs (M-CANONICAL-DOCS), idiomatic patterns (M-DESIGN-FOR-AI), error handling (M-APP-ERROR with anyhow)

### Testing Standards (NON-NEGOTIABLE)
PASS:
- Plan includes unit tests for visible-text extraction, inline-reference normalization, filename planning, and rewrite behavior.
- Plan includes integration coverage for end-to-end keyword filtering and exported HTML/attachment layout.
- New behavior is framed as regression coverage for current false-positive HTML matching and broken inline references.

### User Experience Consistency
PASS:
- No new CLI flags are introduced; existing `--keywords`, `--attachments`, and `--metadata` workflows keep their current shape.
- Export output remains human-readable and deterministic, with rewritten references using the same attachment filenames users already see on disk.

### Performance Requirements
PASS:
- HTML parsing occurs only for HTML bodies and is bounded to per-message content already loaded in memory.
- Attachment filename planning is reused for both writes and HTML rewriting, avoiding redundant collision resolution logic.

### Post-Design Gate Re-Check

PASS:
- `research.md` resolves HTML parsing, rewrite engine, attachment metadata, and filename planning decisions.
- `data-model.md` defines the message, attachment, and inline-reference entities needed for implementation.
- `contracts/cli-interface.md` describes user-visible filtering and export behavior without adding new flags.
- `quickstart.md` documents validation flows and expected output.

**Constitution Compliance**: All gates PASS. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/003-html-search-inline-export/
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
│   │   └── mod.rs                  # Existing export flags remain unchanged
│   ├── filter/
│   │   └── keyword.rs              # HTML-visible-text keyword matching
│   └── export/
│       ├── mod.rs                  # Message extraction, keyword filtering, attachment metadata capture
│       ├── exporter.rs             # Attachment filename planning, HTML rewriting, file writes
│       ├── html.rs                 # HTML conversion plus parse-based visible-text/rewrite helpers
│       └── metadata.rs             # Attachment names/metadata consistency if needed
└── tests/
    ├── integration/
    │   ├── export_test.rs
    │   └── filtering_test.rs       # End-to-end export and filtering scenarios
    └── unit/
        ├── attachment_test.rs
        ├── html_test.rs
        └── keyword_test.rs         # New regression tests for HTML search/rewrite rules

crates/pst/
└── src/
    └── messaging/
        └── attachment.rs           # Existing raw attachment property access if helper methods are warranted
```

**Structure Decision**: Keep the feature inside the current `pst-cli` pipeline, extending the existing `MessageData` and `Attachment` flow rather than introducing a new subsystem. Only touch `crates/pst` if a small accessor addition materially improves clarity over direct property reads already available during attachment extraction.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations identified.
