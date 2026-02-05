# Data Model: PST CLI Tool

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [research.md](research.md)  
**Phase**: 1 - Design  
**Date**: 2026-02-05

## Overview

This document defines the core data entities and their relationships for the pst-cli tool. The model supports PST message extraction, duplicate detection, filtering, and export to various output formats.

---

## Core Entities

### PstMessage

Represents an email message extracted from a PST file.

**Properties**:
- `message_id`: `Option<String>` - RFC 2822 Message-ID header (if present)
- `subject`: `String` - Email subject line
- `from`: `EmailAddress` - Sender information
- `to`: `Vec<EmailAddress>` - Primary recipients
- `cc`: `Vec<EmailAddress>` - Carbon copy recipients
- `bcc`: `Vec<EmailAddress>` - Blind carbon copy recipients
- `date`: `DateTime<Utc>` - Message timestamp (parsed from Date header)
- `body_html`: `Option<String>` - HTML body content (if present)
- `body_rtf`: `Option<Vec<u8>>` - Compressed RTF body (if present)
- `body_plain`: `Option<String>` - Plain text body (if present)
- `headers`: `HashMap<String, String>` - All transport headers
- `attachments`: `Vec<Attachment>` - Message attachments
- `folder_path`: `String` - Internal PST folder path (e.g., "Inbox/Archive/2023")
- `size_bytes`: `u64` - Total message size in bytes
- `flags`: `MessageFlags` - PST message flags (read, flagged, etc.)

**Validation Rules**:
- At least one body format MUST be present (html, rtf, or plain)
- `date` MUST be valid RFC 3339 timestamp
- `from` MUST contain at least email address (display name optional)
- `folder_path` MUST be non-empty

**Relationships**:
- Source: `PstFileSource` (many PstMessage : one PstFileSource)
- Exports to: `ExportItem` (one PstMessage : one ExportItem)

---

### EmailAddress

Represents an email participant (sender or recipient).

**Properties**:
- `address`: `String` - Email address (e.g., "user@example.com")
- `display_name`: `Option<String>` - Human-readable name (e.g., "John Doe")

**Validation Rules**:
- `address` MUST be valid email format (contains @, domain part)
- `address` stored in lowercase for case-insensitive comparisons

**Derived Properties**:
- `formatted()`: `String` - Returns "Display Name <address>" or just "address" if no display name

---

### Attachment

Represents a file attached to an email message.

**Properties**:
- `filename`: `String` - Original attachment filename
- `content_type`: `Option<String>` - MIME type (if available)
- `size_bytes`: `u64` - Attachment size
- `data`: `Vec<u8>` - Attachment binary content

**Validation Rules**:
- `filename` MUST be non-empty
- `size_bytes` MUST match `data.len()`

**Derived Properties**:
- `safe_filename()`: `String` - Filesystem-safe version of filename (ASCII, no path separators)

---

### MessageFlags

Binary flags representing message state in PST.

**Fields**:
- `read`: `bool` - Message has been read
- `flagged`: `bool` - Message is flagged/starred
- `replied`: `bool` - Message has been replied to
- `forwarded`: `bool` - Message has been forwarded
- `draft`: `bool` - Message is a draft

**Display Format**:
```
[Read, Flagged] or [Unread] etc.
```

---

### ExportItem

Represents an exported message with assigned output location and metadata.

**Properties**:
- `sequence_number`: `u32` - Sequential export number (00001, 00002, etc.)
- `message`: `PstMessage` - Associated source message
- `output_dir`: `PathBuf` - Export subdirectory path (e.g., "output/00001/")
- `files_created`: `Vec<PathBuf>` - List of created files (message.html, metadata.txt, etc.)
- `is_duplicate`: `bool` - True if message is a duplicate
- `duplicate_of`: `Option<u32>` - Sequence number of first occurrence (if duplicate)
- `content_hash`: `String` - SHA-256 hash for duplicate detection (Message-ID or fallback)
- `keyword_matches`: `Vec<String>` - Matched keywords (when --keywords used)
- `email_matches`: `Vec<EmailAddress>` - Matched email participants (when --emails used)

**Validation Rules**:
- `sequence_number` MUST be unique within export
- If `is_duplicate` is true, `duplicate_of` MUST be Some with valid sequence number
- `output_dir` naming MUST match `{sequence_number:05}` format

**Relationships**:
- Derived from: `PstMessage` (one ExportItem : one PstMessage)
- Tracked by: `DuplicateTracker`

---

### PstFileSource

Represents a source PST file being processed.

**Properties**:
- `file_path`: `PathBuf` - Absolute path to PST file
- `file_name`: `String` - PST filename (for display/sorting)
- `file_size_bytes`: `u64` - PST file size
- `message_count`: `usize` - Total messages in this PST
- `folder_structure`: `Vec<FolderInfo>` - Folder hierarchy (for list command)
- `processing_status`: `ProcessingStatus` - Current processing state

**Validation Rules**:
- `file_path` MUST exist and be readable
- `file_path` MUST have .pst extension
- Multiple PST files processed in alphabetical order of `file_name`

**Relationships**:
- Contains: `PstMessage` (one PstFileSource : many PstMessage)

---

### FolderInfo

Represents a folder within PST hierarchy (used by list command).

**Properties**:
- `name`: `String` - Folder name
- `path`: `String` - Full folder path (e.g., "Inbox/Archive/2023")
- `message_count`: `usize` - Number of messages in this folder
- `subfolder_count`: `usize` - Number of subfolders

**Display Format**:
```
Inbox (45 messages, 2 subfolders)
  ├── Archive (12 messages, 1 subfolder)
  │   └── 2023 (8 messages)
  └── Important (5 messages)
```

---

### ProcessingStatus

Enum representing PST file processing state.

**Variants**:
- `Pending` - Not yet started
- `InProgress { current_message: usize, total: usize }` - Currently processing
- `Completed { messages_exported: usize, errors: usize }` - Finished successfully
- `Failed { error: String }` - Failed with error

---

### DuplicateTracker

Tracks message identifiers to detect duplicates across all PST files.

**Properties**:
- `seen_hashes`: `HashMap<String, u32>` - Map of content hash → first occurrence sequence number
- `duplicate_count`: `usize` - Total duplicates found

**Methods**:
- `check_duplicate(hash: String, seq_num: u32) -> DuplicateStatus` - Check if hash seen before
- `register_message(hash: String, seq_num: u32)` - Register new message hash

**DuplicateStatus Enum**:
- `Unique` - First occurrence of this message
- `Duplicate(u32)` - Duplicate; returns sequence number of first occurrence

---

### KeywordMatcher

Performs case-insensitive keyword matching in message subject and body.

**Properties**:
- `keywords`: `HashSet<String>` - Normalized keywords (lowercase)

**Methods**:
- `find_matches(message: &PstMessage) -> Vec<String>` - Returns matched keywords
- Searches in: `message.subject`, `message.body_plain()`, `message.body_html_text()`

**Matching Rules**:
- Case-insensitive
- Whole-word or substring matching (configurable in research phase)
- Returns each keyword at most once per message (presence, not count)

---

### EmailMatcher

Performs case-insensitive email address matching across message participants.

**Properties**:
- `target_emails`: `HashSet<String>` - Normalized email addresses (lowercase)

**Methods**:
- `find_matches(message: &PstMessage) -> Vec<EmailAddress>` - Returns matched participants
- Searches in: `message.from`, `message.to`, `message.cc`, `message.bcc`

**Matching Rules**:
- Case-insensitive on email address only (ignore display name)
- Returns each unique matched address once per message

---

### ExportStatistics

Aggregated statistics for export operation (displayed after completion).

**Properties**:
- `total_messages`: `usize` - Total messages processed
- `unique_messages`: `usize` - Non-duplicate messages
- `duplicate_messages`: `usize` - Duplicate messages
- `errors_encountered`: `usize` - Messages that failed to export
- `messages_with_keywords`: `usize` - Messages matching any keyword (when --keywords used)
- `messages_with_email_matches`: `usize` - Messages with matching participants (when --emails used)
- `files_created`: `usize` - Total output files created
- `elapsed_time`: `Duration` - Total processing time

**Methods**:
- `display_summary()` - Print formatted summary to stderr (unless --quiet)

---

### CsvRow

Represents one row in the CSV summary export (emails.csv).

**Properties**:
- `sequence_number`: `u32`
- `message_id`: `Option<String>`
- `subject`: `String`
- `from`: `String` - Formatted email address
- `to`: `String` - Comma-separated list of recipients
- `cc`: `String` - Comma-separated list
- `bcc`: `String` - Comma-separated list
- `date`: `String` - ISO 8601 format
- `folder_path`: `String`
- `size_bytes`: `u64`
- `attachment_count`: `usize`
- `attachment_names`: `String` - Comma-separated list
- `is_duplicate`: `bool`
- `keyword_count`: `usize` - Number of matched keywords (0 if no keywords)
- `email_match_count`: `usize` - Number of matched email participants (0 if no emails)

**CSV Header**:
```
SequenceNumber,MessageID,Subject,From,To,CC,BCC,Date,FolderPath,SizeBytes,AttachmentCount,AttachmentNames,IsDuplicate,KeywordCount,EmailMatchCount
```

**Escaping Rules**:
- Use `csv` crate standard escaping for commas, quotes, newlines
- Multi-line fields (subject, body snippets) quoted and newlines preserved

---

## Entity Relationships Diagram

```
PstFileSource
   │
   │ contains (1:N)
   ▼
PstMessage ──────┐
   │             │
   │ exports to  │ checked by
   │ (1:1)       │ (N:1)
   ▼             ▼
ExportItem   DuplicateTracker
   │
   │ aggregated by (N:1)
   ▼
ExportStatistics
   │
   │ exported to (N:1)
   ▼
CsvRow (in emails.csv)
```

**Filtering (optional flow)**:
```
PstMessage ──> KeywordMatcher ──> matched keywords stored in ExportItem
           └──> EmailMatcher   ──> matched emails stored in ExportItem
```

---

## State Transitions

### Export Operation Flow

1. **Initialize**: Create `DuplicateTracker`, `ExportStatistics`, optional `KeywordMatcher`/`EmailMatcher`
2. **Discover PST Files**: Scan input directory, create `PstFileSource` for each .pst file
3. **Sort PST Files**: Alphabetical order of `file_name` (deterministic processing)
4. **For Each PST File**:
   - Open PST, iterate folders and messages (streaming)
   - For each `PstMessage`:
     - Generate content hash (Message-ID or SHA-256 fallback)
     - Check `DuplicateTracker` for duplicate status
     - Assign sequence number (global counter across all PSTs)
     - Create `ExportItem` (mark as duplicate if applicable)
     - Apply filters (keywords/emails) if enabled
     - Export to output directory:
       - Always: `message.html`
       - If `--metadata`: `metadata.txt`
       - If `--attachments`: Save attachments
       - If `--headers`: `headers.txt`
       - If error: `error.txt` + partial content
     - Update progress counter (stderr)
     - Aggregate `ExportStatistics`
5. **Finalize**:
   - If `--csv`: Generate `emails.csv` from all `ExportItem`s
   - Display `ExportStatistics.display_summary()` (unless `--quiet`)

---

## Data Persistence

### Input
- **PST Files**: Read-only, streamed from disk via `pst` crate

### Output (File System)
```
<output_dir>/
├── 00001/
│   ├── message.html      (always)
│   ├── metadata.txt      (if --metadata)
│   ├── headers.txt       (if --headers)
│   ├── error.txt         (if partial export due to error)
│   └── attachment1.pdf   (if --attachments)
├── 00002/
│   └── ...
├── duplicates/
│   └── 00010/
│       └── message.html
└── emails.csv            (if --csv, at root)
```

### In-Memory (During Export)
- `DuplicateTracker.seen_hashes`: HashMap<String, u32> - grows with unique messages
- `ExportStatistics`: Scalar counters
- Optional: `Vec<CsvRow>` for generating CSV at end (alternative: stream CSV rows as messages processed to save memory)

**Memory Optimization**: For CSV, stream rows to file incrementally rather than buffering all rows in memory (important for 10K+ message exports).

---

## Validation & Invariants

### Global Invariants
- **Deterministic numbering**: Same PST files processed in same order always produce same sequence numbers
- **No duplicate sequence numbers**: Each `ExportItem.sequence_number` is unique
- **Atomic exports**: Each message either fully exported or recorded as error (no silent partial failures)
- **Content hash uniqueness**: Each unique content hash maps to one primary sequence number

### Data Quality Checks
- **Message-ID validation**: If present, must be non-empty string
- **Date parsing**: Invalid dates fall back to PST file modification time or null (documented in error.txt)
- **Email address validation**: Basic format check (contains @), invalid addresses logged but don't fail export
- **Attachment filename sanitization**: Special characters removed/replaced, collisions handled with numeric suffixes

---

## Summary

This data model supports:
✅ Streaming PST processing (no full file load)  
✅ Duplicate detection across files  
✅ Optional filtering (keywords, emails)  
✅ Multi-format export (HTML, CSV, metadata, attachments)  
✅ Error resilience (partial exports with error logs)  
✅ Observable progress and statistics  

**Key Design Principles**:
- **Streaming-first**: Entities designed for incremental processing  
- **Error isolation**: Bad message doesn't fail entire export  
- **Memory-efficient**: Minimal state retained across messages  
- **Testable**: Clear entity boundaries enable unit testing  

**Next**: Define CLI contracts in [contracts/cli-interface.md](contracts/cli-interface.md)
