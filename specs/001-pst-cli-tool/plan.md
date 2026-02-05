# Implementation Plan: PST CLI Tool

**Branch**: `001-pst-cli-tool` | **Date**: 2026-02-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-pst-cli-tool/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Create pst-cli, a command-line tool for eDiscovery workflows that exports PST file contents to HTML with optional metadata, attachments, headers, and CSV summaries. The tool must process PST files deterministically, detect duplicates via Message-ID (or content hash fallback), and support keyword/email filtering. Technical approach: new Rust crate using existing pst and compressed-rtf libraries with subcommand-style CLI (export/list), streaming PST processing for memory efficiency, and configurable output formats with progress reporting and summary statistics.

## Technical Context

**Language/Version**: Rust 1.82 (edition 2021)  
**Primary Dependencies**: pst crate (workspace), compressed-rtf crate (workspace), clap 4.x (CLI parsing with derive), anyhow (application error handling per M-APP-ERROR)  
**Storage**: Files - Input: PST files; Output: HTML files, metadata.txt, headers.txt, attachments, emails.csv  
**Testing**: cargo test with unit tests, integration tests using sample PST fixtures, example validation  
**Target Platform**: Cross-platform (Linux, macOS, Windows) - no OS-specific code per Portability & Correctness principle  
**Project Type**: Single CLI application crate in workspace  
**Performance Goals**: Export 10,000 messages in <10 minutes; stream-process 1GB+ PST files without memory exhaustion (SC-001, SC-009)  
**Constraints**: Deterministic output numbering (repeatable builds); case-insensitive duplicate/keyword/email matching; defensive parsing of untrusted PST data  
**Scale/Scope**: eDiscovery tool processing multiple PST files (hundreds of thousands of messages total); batch processing with error resilience

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality (NON-NEGOTIABLE)
✅ **PASS** - Plan follows Rust idioms and guidelines:
- Safe Rust throughout (no FFI, no unsafe required for CLI and format conversion)
- Strong typing for entities (Message, ExportItem, metadata fields)
- Public API limited to CLI interface only (library code in lib.rs is workspace-internal)
- Follows `rust-guidelines.txt`: comprehensive docs (M-CANONICAL-DOCS), idiomatic patterns (M-DESIGN-FOR-AI), error handling (M-APP-ERROR with anyhow)

### Testing Standards (NON-NEGOTIABLE)
✅ **PASS** - Comprehensive test strategy planned:
- Unit tests: message parsing, duplicate detection, keyword/email matching, HTML conversion
- Integration tests: end-to-end PST export with sample fixtures covering edge cases (corrupted messages, missing headers, various body formats)
- Example-based tests: canonical export scenarios in examples/ directory run by CI
- Benchmarks: export performance (<10 min for 10K messages), memory usage (<streaming 1GB files)

### User Experience Consistency
✅ **PASS** - Consistent CLI UX planned:
- Subcommand style (`pst-cli export`/`pst-cli list`) follows Rust CLI conventions (cargo, rustc pattern)
- Structured error messages with context (e.g., "Failed to parse PST: <file> offset <X>: <reason>")
- Progress indicators (stderr counter) + summary statistics unless --quiet
- Canonical examples for common workflows (basic export, batch processing, filtering)

### Performance Requirements
✅ **PASS** - Explicit performance goals with measurement plan:
- Target: 10,000 messages in <10 minutes (SC-009)
- Memory constraint: stream-process 1GB+ PST without exhaustion (SC-001)
- Benchmarks planned: message export rate, duplicate detection performance, HTML conversion throughput
- Profiling strategy: identify hot paths (PST parsing, HTML generation, duplicate hashing) before optimization

**Constitution Compliance**: All gates PASS. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/001-pst-cli-tool/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── cli-interface.md # CLI command structure and argument specifications
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/pst-cli/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs          # CLI entry point, subcommand dispatch
│   ├── lib.rs           # Public library interface for core logic
│   ├── export/
│   │   ├── mod.rs       # Export command orchestration
│   │   ├── exporter.rs  # PST message export logic
│   │   ├── html.rs      # HTML generation from message bodies
│   │   ├── metadata.rs  # Metadata extraction and formatting
│   │   └── csv.rs       # CSV summary generation
│   ├── list/
│   │   └── mod.rs       # List command implementation
│   ├── duplicate/
│   │   ├── mod.rs       # Duplicate detection coordinator
│   │   └── hash.rs      # Content hash generation for fallback
│   ├── filter/
│   │   ├── keyword.rs   # Keyword matching logic
│   │   └── email.rs     # Email participant matching logic
│   ├── cli/
│   │   ├── mod.rs       # CLI argument parsing (clap structs)
│   │   └── progress.rs  # Progress reporting and summary statistics
│   └── error.rs         # Error types and handling
├── examples/
│   ├── basic_export.rs  # Simple PST export example
│   └── batch_export.rs  # Multi-file processing example
└── tests/
    ├── integration/
    │   ├── export_test.rs       # End-to-end export scenarios
    │   ├── duplicate_test.rs    # Duplicate detection tests
    │   └── filtering_test.rs    # Keyword/email filtering tests
    ├── fixtures/
    │   ├── sample.pst           # Small test PST file
    │   └── with_duplicates.pst  # PST with known duplicates
    └── unit/
        ├── html_conversion_test.rs
        ├── content_hash_test.rs
        └── metadata_test.rs
```

**Structure Decision**: Single CLI application crate at `crates/pst-cli/` within the workspace. Modular internal structure separates export logic, duplicate detection, filtering, and CLI concerns. Integration tests use fixtures/ directory for sample PST files. Examples demonstrate common usage patterns and serve as validation tests.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - all constitution gates PASS. No complexity justification required.
