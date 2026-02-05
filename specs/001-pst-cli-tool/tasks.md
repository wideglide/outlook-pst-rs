# Tasks: PST CLI Tool

**Input**: Design documents from `/specs/001-pst-cli-tool/`  
**Prerequisites**: [plan.md](plan.md) (required), [spec.md](spec.md) (required), [research.md](research.md), [data-model.md](data-model.md), [contracts/cli-interface.md](contracts/cli-interface.md), [quickstart.md](quickstart.md)

**Organization**: Tasks are grouped by user story (P1-P8) to enable independent implementation and testing of each story. All paths are relative to `crates/pst-cli/`.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: User story label for traceability (US1, US2, US3...US8)
- **File paths**: Always included in task description for clarity

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create crates/pst-cli directory structure with src/, examples/, tests/unit, tests/integration, tests/fixtures subdirectories
- [ ] T002 Create crates/pst-cli/Cargo.toml with dependencies: clap 4.x, anyhow, sha2, csv, encoding_rs, and workspace references to pst/compressed-rtf
- [ ] T003 [P] Create crates/pst-cli/README.md describing the tool, installation, basic usage examples
- [ ] T004 [P] Configure rustfmt and clippy in crates/pst-cli with workspace standards

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 [P] Create src/lib.rs with public lib module exports and basic PST file source abstraction
- [ ] T006 [P] Create src/error.rs with comprehensive error types (PstError, ExportError, DuplicateError, FilterError) and Display implementation per M-APP-ERROR principle
- [ ] T007 [P] Create src/cli/mod.rs with clap derive structs for global options (--help, --version, --quiet) and subcommand enum (Export, List)
- [ ] T008 [P] Create src/cli/mod.rs with argument parsing for export command: positional <INPUT>, required --output, optional flags (--metadata, --attachments, --headers, --csv, --keywords, --emails)
- [ ] T009 [P] Create src/cli/progress.rs with ProgressReporter struct for progress indicator ("Processing message N/Total...") and summary statistics struct (total, duplicates, errors, elapsed_time)
- [ ] T010 [P] Create src/export/mod.rs with ExportCoordinator struct managing overall export workflow orchestration and file I/O
- [ ] T011 [P] Create src/list/mod.rs with ListCommand struct for folder traversal stubs (implementation in US8)
- [ ] T012 Create src/main.rs with CLI entry point: parse args via clap, dispatch to export/list subcommands, handle --quiet flag, call progress.summary_statistics() unless --quiet
- [ ] T013 Create crates/pst-cli/examples/simple_usage.rs demonstrating basic export/list command usage for reference
- [ ] T014 [P] Create tests/unit/error_handling_test.rs with unit tests for error type Display formatting per M-APP-ERROR
- [ ] T015 Create tests/fixtures/sample.pst (small test PST file with 5-10 messages for unit/integration testing - or use compressed test data)

**Checkpoint**: Foundation ready - project structure complete, CLI argument parsing functional, error handling in place, progress reporting infrastructure ready. User story implementation can now begin in parallel.

---

## Phase 3: User Story 1 - Basic PST Export to HTML (Priority: P1) 🎯 MVP

**Goal**: Export email messages from a single PST file to individual numbered HTML files containing readable message content converted from the best available body format (HTML > RTF > plain text).

**Independent Test**: Provide a single PST file with 10 messages and verify that 10 HTML files are created in numbered subdirectories (00001/ through 00010/), each containing the expected message content in browser-readable HTML format.

### Implementation for User Story 1

- [ ] T016 [P] [US1] Create PstMessage struct in src/lib.rs with fields: pst_id, subject, from, to, cc, bcc, date, message_id, body_html, body_rtf, body_text, attachments, headers, folder_path, flags
- [ ] T017 [P] [US1] Create EmailAddress struct in src/lib.rs with fields: display_name, email_address
- [ ] T018 [P] [US1] Implement Message reader in src/export/exporter.rs reading PST message data into PstMessage struct using pst crate API
- [ ] T019 [P] [US1] Create src/export/html.rs with function to convert body to HTML (HTML body used directly; RTF decompressed via compressed-rtf crate and converted to HTML; plain text wrapped with basic HTML formatting like <p>, <br>)
- [ ] T020 [P] [US1] Implement character encoding handling in src/export/html.rs to preserve message content accuracy with proper UTF-8 conversion
- [ ] T021 [US1] Implement message folder structure creation in src/export/exporter.rs: create output_dir/NNNNN/ for each message number, verify path writability upfront
- [ ] T022 [US1] Implement message.html file writing in src/export/exporter.rs with proper error handling (export partial content with error markers in HTML if conversion fails, log to stderr and error.txt file)
- [ ] T023 [US1] Implement message numbering (5-digit zero-padded: 00001, 00002) in src/export/mod.rs with global counter across all processed messages
- [ ] T024 [US1] Implement deterministic PST message processing order in src/lib.rs (read messages in folder order, process all messages from first PST before moving to next)
- [ ] T025 [US1] Add --quiet flag support in src/cli/progress.rs to suppress progress indicators and summary statistics (only errors to stderr)
- [ ] T026 [P] [US1] Unit tests for HTML conversion in tests/unit/html_test.rs: HTML pass-through, RTF to HTML (check for basic tags), plain text wrapping (check <p> and <br> tags), encoding handling
- [ ] T027 [P] [US1] Unit tests for message numbering in tests/unit/numbering_test.rs: sequence correctness, zero-padding format, counter state management
- [ ] T028 [US1] Integration test for basic single-file export in tests/integration/export_test.rs: create minimal PST fixture, run export, verify folder structure, verify message.html content correctness
- [ ] T029 [US1] Create example in examples/basic_export.rs demonstrating single PST export to HTML with progress output

**Checkpoint**: User Story 1 is complete and independently testable. HTML export from single PST works correctly. MVP functionality validated.

---

## Phase 4: User Story 2 - Batch Export from Multiple PST Files (Priority: P2)

**Goal**: Process multiple PST files from a directory deterministically (alphabetical order) and export all messages with sequential numbering across all files.

**Independent Test**: Create a directory with 3 PST files containing known message counts (5, 10, 7 respectively) and verify that all 22 messages are exported with unique sequential numbering (00001 through 00022) in deterministic order when export runs multiple times.

### Implementation for User Story 2

- [ ] T030 [P] [US2] Implement directory handling in src/lib.rs: detect if input is directory or file, scan for .pst files (case-sensitive extension on Linux/macOS)
- [ ] T031 [P] [US2] Implement PST file discovery and alphabetical sorting in src/lib.rs to ensure deterministic processing order across multiple runs
- [ ] T032 [US2] Implement cross-file message numbering continuation in src/export/mod.rs: after processing first PST, continue numbering sequence from last number for next PST
- [ ] T033 [US2] Add error handling for invalid PST files in src/lib.rs: log error to stderr, continue processing remaining files in directory without stopping export
- [ ] T034 [US2] Integration test for batch export in tests/integration/export_test.rs: create multi-PST test directory, run export twice, verify identical numbering and output structure both times (deterministic)
- [ ] T035 [US2] Create example in examples/batch_export.rs demonstrating multi-file directory export with progress output

**Checkpoint**: User Stories 1 AND 2 both work. Single file and batch directory processing are functional and produce deterministic results.

---

## Phase 5: User Story 3 - Duplicate Detection by Message ID (Priority: P3)

**Goal**: Identify duplicate messages across all PST files based on Message-ID header, with fallback to content hash (SHA-256 from Subject+Date+From+Body) for messages without Message-ID. Export first occurrence to main output directory and duplicates to subdirectory.

**Independent Test**: Create PST files with known duplicate messages (same Message-ID) and verify that first occurrence is in main output directory while duplicates are correctly placed in duplicates/NNNNN/ subdirectory.

### Implementation for User Story 3

- [ ] T036 [P] [US3] Create DuplicateTracker struct in src/lib.rs with HashMap<String, u32> (hash/message_id -> sequence_number) and method to check if message is duplicate
- [ ] T037 [P] [US3] Implement Message-ID extraction in src/duplicate/mod.rs: extract from message headers, normalize, handle missing Message-ID by returning empty string for fallback logic
- [ ] T038 [P] [US3] Create src/duplicate/hash.rs with function to generate content hash (SHA-256 from Subject+Date+From+Body fields concatenated and hashed)
- [ ] T039 [US3] Implement duplicate detection logic in src/duplicate/mod.rs: check Message-ID first, fallback to content hash if Message-ID missing, store identifier in DuplicateTracker, return whether message is first occurrence or duplicate
- [ ] T040 [US3] Implement duplicates/ directory structure in src/export/exporter.rs: if message is duplicate, write to output_dir/duplicates/NNNNN/ instead of output_dir/NNNNN/
- [ ] T041 [US3] Integrate DuplicateTracker into export pipeline in src/export/mod.rs: track duplicates across all messages, pass duplicate status to exporter
- [ ] T042 [P] [US3] Unit tests for Message-ID extraction in tests/unit/duplicate_test.rs: empty Message-ID handling, normalization, special character handling
- [ ] T043 [P] [US3] Unit tests for content hash generation in tests/unit/duplicate_test.rs: hash consistency (same input = same hash), different messages have different hashes
- [ ] T044 [US3] Integration test for duplicate detection in tests/integration/duplicate_test.rs: PST with known duplicates, verify first occurrence in main dir, duplicate in duplicates/ dir, no false positives/negatives

**Checkpoint**: Duplicate detection across files is functional. Messages correctly separated into main/duplicates directories. Fallback hashing works for messages without Message-ID.

---

## Phase 6: User Story 4 - Enhanced Export with Metadata and Attachments (Priority: P4)

**Goal**: When optional flags are provided (--metadata, --attachments, --headers), export additional information alongside HTML: metadata.txt (Subject, From, To, CC, BCC, Date, MessageId, Folder, Size, Attachments, Flags), attachments files with original names, headers.txt with full transport headers.

**Independent Test**: Export a message with attachments and metadata flags enabled, verify all three files (message.html, metadata.txt, headers.txt) are created with expected fields, attachments have correct filenames and content.

### Implementation for User Story 4

- [ ] T045 [P] [US4] Create src/export/metadata.rs with MetadataExtractor struct to extract fields from PstMessage (Subject, From display_name and address, To/CC/BCC, Date, MessageId, Folder, Size, Attachment names, Flags)
- [ ] T046 [P] [US4] Implement metadata.txt formatting in src/export/metadata.rs with human-readable layout and handling for missing fields (show "N/A" or empty)
- [ ] T047 [P] [US4] Implement attachment extraction in src/export/exporter.rs: get attachment data from PstMessage, write each to message folder with original filename (or sanitized)
- [ ] T048 [P] [US4] Implement filename sanitization for attachments in src/export/exporter.rs: replace filesystem-unsafe characters with underscores or safe alternatives, preserve file extensions
- [ ] T049 [US4] Add --metadata flag handling in src/cli/mod.rs: parse flag, pass to export coordinator
- [ ] T050 [US4] Add --attachments flag handling in src/cli/mod.rs: parse flag, pass to export coordinator
- [ ] T051 [US4] Add --headers flag handling in src/cli/mod.rs: parse flag, pass to export coordinator
- [ ] T052 [US4] Implement headers.txt generation in src/export/metadata.rs: extract full transport headers from PstMessage, write to headers.txt preserving order and formatting
- [ ] T053 [US4] Implement attachment collision handling in src/export/exporter.rs: if two attachments have same name in same message, add numeric suffix (e.g., document_1.pdf, document_2.pdf)
- [ ] T054 [US4] Integrate metadata/attachment/headers generation into export pipeline in src/export/exporter.rs: only generate files if corresponding flags are set
- [ ] T055 [P] [US4] Unit tests for metadata extraction in tests/unit/metadata_test.rs: field extraction, missing field handling, display name/address parsing
- [ ] T056 [P] [US4] Unit tests for filename sanitization in tests/unit/metadata_test.rs: special character replacement, extension preservation
- [ ] T057 [US4] Integration test for metadata/attachments/headers export in tests/integration/export_test.rs: run with all flags, verify all files present with correct content

**Checkpoint**: Metadata, attachments, and headers export fully functional. Optional flags work correctly and don't interfere with core export flow.

---

## Phase 7: User Story 5 - CSV Summary Export (Priority: P5)

**Goal**: When --csv flag is provided, create emails.csv in output directory root with one row per exported message. CSV includes key metadata columns (SequenceNumber, Subject, From, To, Date, MessageId, IsDuplicate, KeywordCount, EmailMatchCount) with proper escaping and valid CSV formatting.

**Independent Test**: Export messages with --csv flag and verify emails.csv can be opened in Excel/LibreOffice without errors, contains correct number of rows (one per message), and all data is properly formatted and escaped.

### Implementation for User Story 5

- [ ] T058 [P] [US5] Create src/export/csv.rs with CsvExporter struct managing CSV file creation and row writing
- [ ] T059 [P] [US5] Implement CSV header row generation in src/export/csv.rs: SequenceNumber, Subject, From, To, Date, MessageId, IsDuplicate, KeywordCount, EmailMatchCount columns
- [ ] T060 [P] [US5] Implement CSV row formatting in src/export/csv.rs with proper escaping (comma, quote, newline characters per CSV standard)
- [ ] T061 [US5] Add --csv flag handling in src/cli/mod.rs: parse flag, pass to export coordinator
- [ ] T062 [US5] Integrate CSV generation into export pipeline in src/export/mod.rs: write row after each message processed, open CSV file once, append rows, close after export complete
- [ ] T063 [US5] Implement duplicate status column in src/export/csv.rs: "true" or "false" based on DuplicateTracker
- [ ] T064 [US5] Prepare columns for keyword_count and email_match_count in src/export/csv.rs (set to 0 initially, will be populated by US6/US7)
- [ ] T065 [P] [US5] Unit tests for CSV formatting in tests/unit/csv_test.rs: proper escaping of special characters, header generation, format validation
- [ ] T066 [US5] Integration test for CSV export in tests/integration/export_test.rs: export with --csv, verify CSV file exists, valid format, correct row count

**Checkpoint**: CSV export fully functional and produces valid, properly-formatted spreadsheet files with metadata summary.

---

## Phase 8: User Story 6 - Keyword Filtering for Relevance (Priority: P6)

**Goal**: When --keywords flag is provided with comma-separated list (e.g., "confidential,merger"), perform case-insensitive search in message subject and body. Report matched keywords in metadata.txt and include keyword_count column in CSV showing number of distinct matched keywords per message.

**Independent Test**: Provide keyword list and messages with varying keyword occurrences and verify matches are found (case-insensitive), counts are correct, and reporting in metadata/CSV is accurate with no false positives.

### Implementation for User Story 6

- [ ] T067 [P] [US6] Create src/filter/keyword.rs with KeywordMatcher struct managing keyword matching logic
- [ ] T068 [P] [US6] Implement case-insensitive keyword search in src/filter/keyword.rs: search message subject and body independently, return set of matched keywords (presence only, not count of occurrences)
- [ ] T069 [P] [US6] Implement keyword parsing in src/filter/keyword.rs: parse comma-separated list, trim whitespace, lowercase, de-duplicate
- [ ] T070 [US6] Add --keywords flag handling in src/cli/mod.rs: parse comma-separated keyword list, pass to export coordinator
- [ ] T071 [US6] Integrate keyword matching into export pipeline in src/export/exporter.rs: call KeywordMatcher for each message, pass matched keywords to metadata/CSV extraction
- [ ] T072 [US6] Implement keyword reporting in metadata.txt in src/export/metadata.rs: list matched keywords or "none" if no matches
- [ ] T073 [US6] Implement keyword_count column population in src/export/csv.rs: count of distinct matched keywords for each message
- [ ] T074 [P] [US6] Unit tests for keyword matching in tests/unit/keyword_test.rs: case-insensitivity, multi-keyword search, presence vs count behavior, missing field handling
- [ ] T075 [US6] Integration test for keyword filtering in tests/integration/filtering_test.rs: keyword search with known test corpus, verify accuracy and reporting

**Checkpoint**: Keyword filtering fully functional. Case-insensitive search works correctly. Metadata and CSV reporting accurate.

---

## Phase 9: User Story 7 - Email Participant Filtering (Priority: P7)

**Goal**: When --emails flag is provided with comma-separated email address list, search From, To, CC, BCC fields (case-insensitive on address portion only, ignoring display names). Report matched email addresses in metadata.txt and include email_match_count column in CSV showing number of distinct matched email addresses per message.

**Independent Test**: Provide email address list and messages with target addresses in various fields and verify matches are found, counts are accurate, and no false positives from display name matches.

### Implementation for User Story 7

- [ ] T076 [P] [US7] Create src/filter/email.rs with EmailMatcher struct managing email participant matching logic
- [ ] T077 [P] [US7] Implement email address extraction in src/filter/email.rs: parse From/To/CC/BCC fields, normalize to email address (extract from "Display Name <address@domain>" format)
- [ ] T078 [P] [US7] Implement case-insensitive email matching in src/filter/email.rs: match on address portion only, case-insensitive, return set of matched addresses
- [ ] T079 [P] [US7] Implement email list parsing in src/filter/email.rs: comma-separated addresses, normalize to lowercase, validate email format, de-duplicate
- [ ] T080 [US7] Add --emails flag handling in src/cli/mod.rs: parse comma-separated email list, pass to export coordinator
- [ ] T081 [US7] Integrate email matching into export pipeline in src/export/exporter.rs: call EmailMatcher for each message, pass matched addresses to metadata/CSV extraction
- [ ] T082 [US7] Implement email participant reporting in metadata.txt in src/export/metadata.rs: list matched email addresses or "none" if no matches
- [ ] T083 [US7] Implement email_match_count column population in src/export/csv.rs: count of distinct matched email addresses for each message
- [ ] T084 [P] [US7] Unit tests for email matching in tests/unit/email_test.rs: address extraction from display name format, case-insensitivity, multi-field search, de-duplication
- [ ] T085 [US7] Integration test for email filtering in tests/integration/filtering_test.rs: email search with known test corpus, verify accuracy and reporting

**Checkpoint**: Email participant filtering fully functional. Address extraction and matching work correctly. Metadata and CSV reporting accurate.

---

## Phase 10: User Story 8 - List PST Contents (Priority: P8)

**Goal**: Provide `pst-cli list <pst-file>` command that displays PST folder structure and message counts per folder. Execution should be fast without loading full message content, providing analyst ability to preview PST contents before export.

**Independent Test**: Run list command on PST with known folder structure and verify output shows correct hierarchy and message counts for each folder quickly.

### Implementation for User Story 8

- [ ] T086 [P] [US8] Implement folder hierarchy traversal in src/list/mod.rs: use pst crate API to walk folder tree, collect folder metadata (path, message count)
- [ ] T087 [P] [US8] Implement message counting per folder in src/list/mod.rs: count messages in each folder without loading message content
- [ ] T088 [US8] Implement folder structure formatting and display in src/list/mod.rs: tree-style output with indentation, message counts per folder, total counts
- [ ] T089 [US8] Implement fast processing in src/list/mod.rs: skip loading message bodies/attachments, only read folder structure
- [ ] T090 [US8] Integrate list command into main.rs: parse list subcommand, create ListCommand, output results to stdout
- [ ] T091 [P] [US8] Unit tests for folder traversal in tests/unit/list_test.rs: hierarchy correctness, count accuracy, formatting
- [ ] T092 [US8] Integration test for list command in tests/integration/list_test.rs: run on sample PST, verify output format and accuracy

**Checkpoint**: List command fully functional. Analysts can preview PST structure quickly before exporting large files.

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Improvements affecting multiple user stories, documentation, testing completeness, and performance validation.

- [ ] T093 [P] Create comprehensive README.md in crates/pst-cli/ with feature overview, installation, basic usage, flag combinations, troubleshooting
- [ ] T094 [P] Add detailed code documentation in src/lib.rs, src/export/mod.rs, src/duplicate/mod.rs, src/filter/keyword.rs with public API and internal structure descriptions
- [ ] T095 [P] Add module-level docstrings in all src/**/*.rs files explaining purpose and key structures
- [ ] T096 [P] Create usage examples in examples/: basic_export.rs, batch_export.rs, filtering_examples.rs demonstrating common workflows from quickstart.md
- [ ] T097 Create performance benchmarks in tests/bench/: 1K message export, 10K message export, duplicate detection performance, measure against targets (<10 min for 10K messages)
- [ ] T098 Create memory profiling test in tests/bench/ for large PST file streaming: load 1GB+ PST without memory exhaustion
- [ ] T099 Run full integration test suite in tests/integration/ covering all user story combinations and edge cases
- [ ] T100 [P] Security review of error messages in src/error.rs: verify no sensitive paths are exposed, appropriate error context provided
- [ ] T101 [P] Security review of filename sanitization in src/export/exporter.rs: verify path traversal vulnerabilities prevented
- [ ] T102 Cross-user story integration testing: ensure US1-US8 work correctly together, no state leaks between stories
- [ ] T103 End-to-end validation test running all example scenarios from quickstart.md
- [ ] T104 [P] Code cleanup: remove debug statements, unused imports, apply clippy suggestions
- [ ] T105 [P] Final refactoring: consolidate duplicated code, improve error message clarity, optimize hot paths identified by profiling
- [ ] T106 Update Cargo.toml metadata: description, authors, license, repository, categories, keywods for discoverability
- [ ] T107 Create CHANGELOG.md documenting initial release (0.1.0) features
- [ ] T108 Final git commit with comprehensive commit message summarizing all completed features

**Checkpoint**: Feature complete, well-tested, documented, and ready for release.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - **BLOCKS all user stories**
- **User Stories (Phase 3-10)**: 
  - All depend on Foundational (Phase 2) completion
  - Can proceed in parallel by priority (P1 → P2 → P3...)
  - Each story is independently testable and delivers value
- **Polish (Phase 11)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Extends P1 but independently testable
- **User Story 3 (P3)**: Can start after Foundational - Works independently but enhances P1+P2
- **User Story 4 (P4)**: Can start after Foundational - Works independently, enhances P1+P2+P3
- **User Story 5 (P5)**: Can start after Foundational - Works independently, provides data from P1-P4
- **User Story 6 (P6)**: Can start after Foundational - Works independently, augments P5
- **User Story 7 (P7)**: Can start after Foundational - Works independently, augments P5
- **User Story 8 (P8)**: Can start after Foundational - Completely independent from P1-P7

### Within Each User Story

1. Read specification and acceptance scenarios first
2. For development: Models → Infrastructure (if needed) → Core logic → Integration
3. For testing: Write tests first (TDD), ensure they FAIL before implementation
4. Follow task numbering order within story
5. Complete story and validate independently before moving to next priority

### Parallel Opportunities

**Immediate Parallelization (After Phase 2)**:
- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel within Phase 2
- Once Foundational completes, all user stories (P1-P8) can start in parallel

**Within Each User Story**:
- All [P] tasks (different files, no dependencies) can run in parallel
- Example: US1 can parallelize HTML conversion, numbering logic, encoding handling simultaneously

**Example: 3-Developer Team Execution**

```
Developer A          Developer B          Developer C
═══════════════════  ═══════════════════  ═══════════════════
Phase 1 (Setup)      Phase 1 (Setup)      Phase 1 (Setup)
Phase 2 (Foundation) Phase 2 (Foundation) Phase 2 (Foundation)
US1 (P1) ────────→   US2 (P2) ────────→   US3 (P3) ────────→
  T016 [P]             T030 [P]             T036 [P]
  T017 [P]             T031 [P]             T037 [P]
  T018 [P]             T032                 T038 [P]
  T019 [P]             T033                 T039
  T020 [P]             T034                 T040
  T021                 T035                 T041
  T022                                      T042
  T023                                      T043 [P]
  T024                                      T044
  T025
  T026 [P]
  T027 [P]
  T028
  T029
```

---

## Implementation Strategy

### MVP First (User Story 1 Only) ← RECOMMENDED

1. Complete Phase 1: Setup (2-4 hours)
2. Complete Phase 2: Foundational (4-6 hours)
3. Complete Phase 3: User Story 1 (6-8 hours)
4. **STOP and VALIDATE**: Test User Story 1 independently - HTML export works perfectly
5. Deploy/demo MVP with basic PST export capability
6. **Then proceed** to P2, P3, etc. in priority order

**MVP Scope**: Single PST files exported to numbered HTML folders with progress reporting. Meets core eDiscovery need.

**Time Estimate**: 12-18 hours for MVP (1-2 developer days)

### Incremental Delivery

1. Phases 1-2 + US1 → MVP released (basic export)
2. Add US2 → Batch processing support
3. Add US3 → Duplicate detection
4. Add US4 → Metadata/attachments/headers
5. Add US5 → CSV summaries
6. Add US6-7 → Filtering capabilities
7. Add US8 → List preview command
8. Phase 11 → Polish, docs, performance optimization

**Each release adds value without breaking previous functionality**

### Parallel Team Strategy (4+ Developers)

1. **Subset A**: Phases 1-2 (shared infrastructure)
2. Once Subset A complete:
   - **Developer 1**: US1 (basic export)
   - **Developer 2**: US2 (batch processing)
   - **Developer 3**: US3 (duplicate detection)
   - **Developer 4**: US4 (metadata/attachments)
3. Once US1-4 complete, proceed with US5-8 in parallel
4. **Final Phase**: US1-8 + Polish (all developers consolidate)

---

## Validation Checkpoints

Validate at these checkpoints to ensure independent story functionality:

- **After Phase 2**: Can run `pst-cli --help` and see available commands
- **After US1**: Can export single PST file to numbered HTML folders
- **After US2**: Can process multiple PST files with consistent numbering
- **After US3**: Duplicates correctly identified and separated
- **After US4**: Metadata, attachments, headers exported alongside HTML
- **After US5**: Valid CSV file generated with all messages and columns
- **After US6**: Keyword filtering works with accurate reporting
- **After US7**: Email participant filtering works with accurate reporting
- **After US8**: List command displays folder structure and counts quickly
- **After Phase 11**: All examples in quickstart.md work correctly

---

## Notes

- **[P] tasks**: Different files with no dependencies - safe to parallelize
- **[Story] labels**: US1-US8 map to user stories in spec.md for traceability
- **File paths**: All relative to `crates/pst-cli/` - adjust as needed for actual checkout
- **Tests first**: Write tests and verify they FAIL before implementing
- **Each story independently testable**: Can validate story without downstream stories
- **Commit strategy**: Commit after each phase or logical group (Phase 1, Phase 2, US1-complete, etc.)
- **Constitution compliance**: All tasks follow Rust safety (safe code), testing standards (unit+integration), UX consistency (CLI patterns), and performance goals (benchmarked)
- **From quickstart**: Use [quickstart.md](quickstart.md) as acceptance test scenarios for implementation validation

---

## Time Estimates

**Total Implementation Time Estimate**:
- Phase 1 (Setup): 2-4 hours
- Phase 2 (Foundational): 4-6 hours
- Phase 3 (US1 MVP): 6-8 hours
- Phase 4 (US2): 4-5 hours
- Phase 5 (US3): 5-6 hours
- Phase 6 (US4): 6-7 hours
- Phase 7 (US5): 3-4 hours
- Phase 8 (US6): 4-5 hours
- Phase 9 (US7): 4-5 hours
- Phase 10 (US8): 3-4 hours
- Phase 11 (Polish): 4-6 hours
- **Total: 45-62 hours** (6-8 developer days at 8 hours/day, or 3-4 weeks with part-time work)

**MVP Only (Phases 1-2 + US1): 12-18 hours** (1-2 developer days)

---

## Running Tests

After each phase or story completion, run:

```bash
# Unit tests for specific module
cargo test --lib -p pst-cli module_name

# Integration tests
cargo test --test integration -p pst-cli

# All tests
cargo test -p pst-cli

# With logging
RUST_LOG=debug cargo test -p pst-cli -- --nocapture
```

---

## Success Definition

Feature is complete when:
1. ✅ All 108 tasks completed
2. ✅ All tests passing (unit + integration + example validation)
3. ✅ All 10 success criteria (SC-001 through SC-010) validated
4. ✅ quickstart.md scenarios tested end-to-end
5. ✅ Performance benchmarks meet targets (<10 min for 10K messages)
6. ✅ Code passes clippy linting and rustfmt formatting
7. ✅ Documentation complete and accurate
8. ✅ Constitution check: Code Quality, Testing Standards, UX Consistency, Performance Requirements all PASS
