# Feature Specification: PST CLI Tool

**Feature Branch**: `001-pst-cli-tool`  
**Created**: 2026-02-05  
**Status**: Draft  
**Input**: User description: "Develop a new crate pst-cli, a command line tool to work with PST files..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic PST Export to HTML (Priority: P1) 🎯 MVP

An eDiscovery analyst needs to export email messages from a single PST file into a reviewable format. The analyst runs `pst-cli export <pst-file> -o <output-dir>` and the tool exports each email as a numbered HTML file (00001, 00002, etc.) containing the message content in a human-readable format. The HTML is generated from the best available body format (HTML > RTF > plain text) with appropriate conversion.

**Why this priority**: This is the foundation of the tool. Without the ability to export PST messages to a readable format, no other functionality has value. This delivers immediate utility for basic eDiscovery review.

**Independent Test**: Can be fully tested by providing a sample PST file and verifying that numbered HTML files are created in the output directory, with each file containing accurate message content.

**Acceptance Scenarios**:

1. **Given** a single PST file with 10 messages, **When** analyst runs export command with output directory, **Then** 10 subdirectories (00001/ through 00010/) are created, each containing message.html
2. **Given** a PST file with messages containing various character encodings, **When** export runs, **Then** all HTML files display message content correctly without corruption
3. **Given** a PST file with nested folder structure, **When** export runs, **Then** messages are extracted in deterministic order regardless of folder structure
4. **Given** invalid PST file, **When** export runs, **Then** tool reports clear error message and exits gracefully

---

### User Story 2 - Batch Export from Multiple PST Files (Priority: P2)

An analyst has a directory containing multiple PST files from different custodians and needs to export all messages into a single output directory for unified review. The tool processes all PST files deterministically and exports messages to sequentially numbered folders.

**Why this priority**: Real eDiscovery projects involve multiple PST files. This extends P1 to handle realistic workflows without requiring the analyst to manually process each file.

**Independent Test**: Can be tested by creating a directory with multiple PST files and verifying that all messages are exported with unique sequential numbering across all files.

**Acceptance Scenarios**:

1. **Given** directory with 3 PST files containing 5, 10, and 7 messages respectively, **When** analyst runs export with directory input, **Then** 22 total messages are exported in folders 00001 through 00022
2. **Given** directory with PST files processed in alphabetical order, **When** export runs multiple times, **Then** same messages receive same numbering each time (deterministic)
3. **Given** directory with mix of valid and invalid PST files, **When** export runs, **Then** valid files are processed and errors are logged for invalid files without stopping the entire export

---

### User Story 3 - Duplicate Detection by Message ID (Priority: P3)

An analyst needs to identify duplicate messages across multiple PST files to avoid reviewing the same email multiple times. The tool detects duplicates based on Message-ID header and exports them to a separate "duplicates" subdirectory.

**Why this priority**: Duplicate detection is critical for eDiscovery efficiency and accuracy. Without it, analysts waste time reviewing identical messages and may produce inconsistent results.

**Independent Test**: Can be tested by creating PST files with duplicate messages (same Message-ID) and verifying that duplicates are correctly identified and placed in the duplicates directory.

**Acceptance Scenarios**:

1. **Given** 2 PST files each containing the same message with Message-ID "abc123", **When** export runs, **Then** first occurrence is in main output (e.g., 00001/), second is in duplicates/00002/
2. **Given** PST with message lacking Message-ID header, **When** export runs, **Then** message is treated as unique and exported normally
3. **Given** 10 PST files with various duplicate patterns, **When** export runs, **Then** duplicate directory contains only true duplicates with same Message-ID

---

### User Story 4 - Enhanced Export with Metadata and Attachments (Priority: P4)

An analyst needs detailed metadata about each message for review spreadsheets and must preserve attachments for evidentiary purposes. Using optional flags (--metadata, --attachments, --headers), the tool exports additional information alongside the HTML.

**Why this priority**: Metadata and attachments are essential for thorough eDiscovery, but base HTML export (P1) provides core value. This enhances the export without blocking basic functionality.

**Independent Test**: Can be tested independently by running exports with each flag combination and verifying correct output files are created with accurate content.

**Acceptance Scenarios**:

1. **Given** message with 2 attachments, **When** export runs with --attachments flag, **Then** both attachments are saved in same folder as message.html with original filenames
2. **Given** message with standard headers, **When** export runs with --metadata flag, **Then** metadata.txt contains Subject, From, Date, To, CC, BCC, MessageId, Folder, Size, Attachment names, and Flags
3. **Given** message with full transport headers, **When** export runs with --headers flag, **Then** headers.txt contains complete email headers including received chains
4. **Given** message with missing optional fields (no CC/BCC), **When** --metadata flag used, **Then** metadata.txt shows empty/N/A for missing fields but includes all other data
5. **Given** attachment with special characters in filename, **When** --attachments used, **Then** attachment is saved with filesystem-safe name and mapping is documented

---

### User Story 5 - CSV Summary Export (Priority: P5)

An analyst needs a spreadsheet overview of all exported messages for quick filtering, sorting, and reporting. Using the --csv flag, the tool creates a summary CSV file with key metadata for all processed messages.

**Why this priority**: CSV summaries enable efficient review and reporting but are supplementary to the core export functionality. Most value comes from having the messages exported first.

**Independent Test**: Can be tested by exporting messages with --csv flag and verifying that emails.csv contains accurate row for each message with all expected columns.

**Acceptance Scenarios**:

1. **Given** 100 exported messages, **When** export runs with --csv flag, **Then** emails.csv contains 100 rows (plus header) with columns for key metadata
2. **Given** messages with various metadata completeness, **When** CSV generated, **Then** all rows are properly formatted with empty cells for missing fields, proper CSV escaping for special characters
3. **Given** messages in duplicates directory, **When** CSV generated, **Then** CSV includes indicator column showing duplicate status

---

### User Story 6 - Keyword Filtering for Relevance (Priority: P6)

An analyst has specific keywords related to the case (e.g., "confidential", "merger", "project alpha") and needs to identify which messages contain these terms. Using --keywords flag with comma-separated list, the tool performs case-insensitive search in subject and body, reporting matches in metadata and CSV.

**Why this priority**: Keyword filtering is valuable for targeted review but the full export is useful even without filtering. This adds analytical capability on top of the core export.

**Independent Test**: Can be tested by providing keyword list and verifying that matches are correctly identified (case-insensitive) in subject and body, with counts reported in metadata.txt and CSV.

**Acceptance Scenarios**:

1. **Given** keyword list "confidential,merger" and messages containing these terms, **When** export runs with --keywords, **Then** metadata.txt shows matched keywords and CSV has keyword_count column
2. **Given** keyword appears 3 times in body and once in subject, **When** keyword search runs, **Then** keyword is reported (not counted multiple times, just presence)
3. **Given** keywords with mixed case in messages, **When** case-insensitive search runs, **Then** all variations are matched (e.g., "CONFIDENTIAL", "Confidential", "confidential")
4. **Given** message with no keyword matches, **When** export runs, **Then** metadata shows "Keywords: none" and CSV shows 0 in keyword_count

---

### User Story 7 - Email Participant Filtering (Priority: P7)

An analyst is investigating specific custodians or external parties and needs to identify which messages involve particular email addresses. Using --emails flag with comma-separated list, the tool identifies messages where specified addresses appear in From, To, CC, or BCC fields, reporting matches in metadata and CSV.

**Why this priority**: Similar to keyword filtering, this adds analytical capability but is not required for basic export. Useful for custodian-focused review.

**Independent Test**: Can be tested by providing email address list and verifying that matches are found in any participant field (From/To/CC/BCC), with counts in metadata and CSV.

**Acceptance Scenarios**:

1. **Given** email list "john@example.com,jane@example.com", **When** export runs with --emails, **Then** metadata.txt shows matched participants and CSV has email_match_count column
2. **Given** target email in CC field, **When** participant search runs, **Then** match is detected and reported
3. **Given** message with multiple matching participants, **When** search runs, **Then** count reflects number of distinct matched addresses in that message
4. **Given** email addresses with different case or display names, **When** search runs, **Then** matching is based on email address only (case-insensitive), ignoring display names

---

### User Story 8 - List PST Contents (Priority: P8)

An analyst wants to preview the structure and contents of a PST file without exporting everything. Using a list command, the tool displays folder hierarchy and message counts to help the analyst decide what to export.

**Why this priority**: Listing is convenient for exploration but not essential since export provides the same information. Lowest priority as it's a "nice to have" preview feature.

**Independent Test**: Can be tested by running list command on PST with known structure and verifying that output shows correct folder hierarchy and message counts.

**Acceptance Scenarios**:

1. **Given** PST with folder hierarchy (Inbox, Sent Items, Archive/2023), **When** list command runs, **Then** folder structure is displayed with message counts per folder
2. **Given** large PST file, **When** list runs, **Then** summary information is displayed quickly without loading full message content
3. **Given** corrupted PST, **When** list runs, **Then** accessible folders are shown and errors are reported for inaccessible sections

---

### Edge Cases

- What happens when output directory already contains numbered folders? (Tool should detect existing numbers and continue sequence, or error if conflicts exist)
- How does system handle PST files larger than available memory? (Stream processing, not load entire file)
- What happens when output directory is not writable? (Clear error before processing begins)
- How are malformed or corrupted messages handled? (Export partial content with error markers in HTML; create error.txt in message folder; log to stderr; continue processing other messages)
- What happens when message has no Message-ID? (Generate stable content hash from Subject+Date+From+Body fields as fallback identifier for duplicate detection)
- How are extremely large attachments handled? (Size limits configurable, or stream to disk)
- What happens when Subject/From/To fields contain invalid characters for filenames? (Sanitize for filesystem safety)
- How are conflicting flags handled (e.g., mutually exclusive options)? (Clear error message explaining conflict)
- What happens when keyword or email list is empty? (Ignore flag or warn user)
- How are nested PST files handled (PST files attached to messages)? (Out of scope - document limitation)
- What body formats are supported for HTML conversion? (HTML body used directly; RTF body decompressed via compressed-rtf crate and converted; plain text wrapped with basic HTML formatting)
- How is progress reported during long exports? (Simple counter to stderr: "Processing message N/Total..." updated in-place; summary statistics at completion unless --quiet flag provided)
- What information is included in summary statistics? (Total messages processed, duplicates found, errors encountered, elapsed time)

## Requirements *(mandatory)*

### Functional Requirements

**Core Export Functionality**
- **FR-001**: Tool MUST accept either a single PST file path or a directory path as input
- **FR-002**: Tool MUST process multiple PST files in a directory in deterministic alphabetical order
- **FR-003**: Tool MUST export each message to a separate subdirectory numbered with 5-digit zero-padded format (00001, 00002, etc.)
- **FR-004**: Tool MUST create a message.html file in each message subdirectory containing HTML representation of the message, converted from the best available source format (prioritize: HTML body > RTF body > plain text body with formatting preservation)
- **FR-005**: Tool MUST preserve message content accuracy including body text, formatting hints, and character encoding
- **FR-006**: Tool MUST continue processing remaining messages when individual message export fails; export partial content with clear error markers in the HTML; log error details to both stderr and an error.txt file in the message folder

**Duplicate Detection**
- **FR-007**: Tool MUST identify duplicate messages across all PST files based on Message-ID header
- **FR-008**: Tool MUST export duplicate messages to a subdirectory named "duplicates" within the output directory
- **FR-009**: Tool MUST treat first occurrence of a Message-ID as primary and subsequent occurrences as duplicates
- **FR-010**: Tool MUST handle messages without Message-ID headers by generating a stable content hash from Subject+Date+From+Body fields as a fallback identifier for duplicate detection

**Metadata Export**
- **FR-011**: When --metadata flag is provided, tool MUST create metadata.txt in each message subdirectory
- **FR-012**: metadata.txt MUST include: Subject, From, Date, To, CC, BCC, MessageId, Folder (internal PST path), Size, Attachment names (if any), and Flags
- **FR-013**: Tool MUST use the best available representation for metadata fields (decoded display names, parsed addresses)
- **FR-014**: metadata.txt MUST handle missing fields gracefully (show as empty or N/A, not fail)

**Attachment Export**
- **FR-015**: When --attachments flag is provided, tool MUST save all message attachments in the same subdirectory as message.html
- **FR-016**: Attachments MUST be saved with their original filenames when filesystem safe, or sanitized filenames with mapping documentation
- **FR-017**: Tool MUST handle attachments with duplicate names within a message by adding numeric suffixes

**Headers Export**
- **FR-018**: When --headers flag is provided, tool MUST create headers.txt containing full transport headers
- **FR-019**: headers.txt MUST preserve header order and formatting as stored in PST

**CSV Summary**
- **FR-020**: When --csv flag is provided, tool MUST create emails.csv in the output directory root
- **FR-021**: emails.csv MUST contain one row per exported message with columns for key metadata fields
- **FR-022**: CSV MUST properly escape special characters (commas, quotes, newlines) per CSV standard
- **FR-023**: CSV MUST include indicator for duplicate status

**Keyword Filtering**
- **FR-024**: When --keywords flag is provided with comma-separated keyword list, tool MUST search message subject and body (case-insensitive)
- **FR-025**: Matched keywords MUST be reported in metadata.txt (when --metadata used)
- **FR-026**: CSV MUST include keyword_count column showing number of distinct matched keywords per message (when --csv used)

**Email Participant Filtering**
- **FR-027**: When --emails flag is provided with comma-separated email list, tool MUST search From, To, CC, and BCC fields (case-insensitive)
- **FR-028**: Matched email addresses MUST be reported in metadata.txt (when --metadata used)
- **FR-029**: CSV MUST include email_match_count column showing number of distinct matched email addresses per message (when --csv used)

**List Functionality**
- **FR-030**: Tool MUST provide a list subcommand (`pst-cli list <pst-file>`) that displays PST folder structure and message counts
- **FR-031**: List command MUST complete quickly without loading full message content

**Error Handling & Logging**
- **FR-032**: Tool MUST validate input paths before processing and provide clear error messages for invalid inputs
- **FR-033**: Tool MUST log all errors and warnings to stderr while continuing processing when possible
- **FR-034**: Tool MUST provide progress indicators for long-running operations (simple counter to stderr showing current message and total, e.g., "Processing message 1234/5000..." updated in-place) and display summary statistics after completion (total messages processed, duplicates found, errors encountered, elapsed time)
- **FR-034a**: Tool MUST support --quiet flag to suppress progress indicators and summary statistics, outputting only errors to stderr
- **FR-035**: Tool MUST exit with appropriate status code (0 for success, non-zero for errors)

**Integration Requirements**
- **FR-036**: Tool MUST use existing pst crate for PST file parsing without modifying pst crate unless changes significantly improve overall system
- **FR-037**: Tool MUST use existing compressed-rtf crate for RTF decompression when needed
- **FR-038**: Tool MUST be packaged as a new crate named pst-cli in the workspace with subcommand-style CLI (export and list subcommands)

### Key Entities

- **PST Message**: Represents an email message extracted from a PST file, with properties including Subject, From, To, CC, BCC, Date, Message-ID, Body (plain text and/or HTML), Attachments, Headers, Folder path, and Flags
- **Export Item**: Represents an exported message with assigned sequential number, output subdirectory path, associated files (message.html, metadata.txt, headers.txt, attachments), and duplicate status
- **PST File Source**: Represents a source PST file being processed, with properties including file path, folder structure, message count, and processing status
- **Keyword Match**: Represents a keyword found in a message, with properties including keyword text, matched field (subject or body), and case-insensitive match
- **Email Participant Match**: Represents a matched email address in a message, with properties including email address, matched field (From/To/CC/BCC), and normalized address

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Tool can successfully export messages from a 1GB PST file without running out of memory (streaming/incremental processing)
- **SC-002**: Tool processes PST files deterministically - running export twice on same inputs produces identical numbering and output structure
- **SC-003**: Tool accurately preserves message content - 100% of exported HTML files display the complete message body without corruption when opened in a browser
- **SC-004**: Tool correctly identifies duplicates - When given test set with known duplicates, duplicate detection achieves 100% accuracy (no false positives or negatives)
- **SC-005**: Tool exports metadata with high fidelity - Best available representation is used for all metadata fields (proper decoding of encoded headers, display names extracted)
- **SC-006**: Tool handles errors gracefully - Processing continues and completes successfully after encountering corrupted messages, with clear error logs
- **SC-007**: CSV export is valid - Generated CSV files can be opened in Excel/LibreOffice without errors and all data is correctly separated into columns
- **SC-008**: Keyword and email filtering is accurate - Case-insensitive matching achieves 100% recall (finds all matches) and precision (no false matches) in test corpus
- **SC-009**: Performance is acceptable - Tool can export 10,000 messages from multiple PST files in under 10 minutes on standard hardware
- **SC-010**: Tool is suitable for eDiscovery use - Exports contain all information needed for legal review, with accurate metadata and proper preservation of attachments

## Clarifications

### Session 2026-02-05

- Q: Command-line interface structure for export and list functionality? → A: Subcommand style (`pst-cli export <input> -o <dir>` and `pst-cli list <input>`)
- Q: HTML body representation strategy when messages have plain text, HTML, RTF, or combinations? → A: Always convert to HTML from best available source (HTML > RTF > plain text with formatting preservation)
- Q: Duplicate detection fallback for messages without Message-ID header? → A: Generate content hash from Subject+Date+From+Body as fallback identifier
- Q: Error handling and partial export strategy when messages are corrupted? → A: Export partial content with error markers in HTML; log to stderr and error.txt in message folder
- Q: Progress reporting detail level for long-running operations? → A: Simple counter to stderr ("Processing message 1234/5000..." updated in-place)
- Q: Should tool display summary statistics after completion? → A: Yes, display summary statistics after completing processing unless --quiet flag is provided
