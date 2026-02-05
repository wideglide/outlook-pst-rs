# CLI Interface Contract: PST CLI Tool

**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [data-model.md](../data-model.md)  
**Phase**: 1 - Design  
**Date**: 2026-02-05

## Overview

This document defines the command-line interface contract for `pst-cli`, specifying all commands, arguments, flags, exit codes, and output formats. This contract serves as the authoritative specification for CLI implementation and testing.

---

## Global Options

Available for all subcommands:

```
--quiet, -q
    Suppress progress indicators and summary statistics.
    Only errors are written to stderr.
    Default: false (show progress and summary)

--help, -h
    Display help information for the command.

--version, -V
    Display version information.
```

---

## Command: export

Export messages from PST file(s) to HTML and optional additional formats.

### Synopsis

```bash
pst-cli export <INPUT> --output <OUTPUT_DIR> [OPTIONS]
```

### Arguments

```
<INPUT>
    Path to a single PST file or a directory containing PST files.
    If directory is provided, all .pst files are processed in
    alphabetical order (case-insensitive, deterministic).
    
    Required: yes
    Type: path (file or directory)
    Validation:
      - Must exist and be readable
      - If file: must have .pst extension
      - If directory: must contain at least one .pst file
```

### Required Flags

```
--output, -o <OUTPUT_DIR>
    Directory where exported messages will be saved.
    Will be created if it doesn't exist.
    
    Required: yes
    Type: path (directory)
    Validation:
      - Parent directory must be writable
      - If exists, must be a directory (not a file)
```

### Optional Flags

```
--metadata, -m
    Export message metadata to metadata.txt in each message folder.
    Includes: Subject, From, Date, To, CC, BCC, MessageId, Folder,
    Size, Attachment names, Flags.
    
    Default: false

--attachments, -a
    Export message attachments to message folder.
    Attachments saved with original filenames (sanitized for filesystem).
    
    Default: false

--headers
    Export full transport headers to headers.txt in each message folder.
    
    Default: false

--csv
    Generate emails.csv summary file in output directory root.
    Contains one row per message with key metadata fields.
    
    Default: false

--keywords <KEYWORD_LIST>
    Comma-separated list of keywords to search for in message
    subject and body (case-insensitive matching).
    Matched keywords reported in metadata.txt and CSV.
    
    Type: comma-separated string
    Example: --keywords "confidential,merger,project alpha"
    
    Default: none (no keyword filtering)

--emails <EMAIL_LIST>
    Comma-separated list of email addresses to search for in
    message participants (From, To, CC, BCC).
    Case-insensitive matching on email address only.
    Matched emails reported in metadata.txt and CSV.
    
    Type: comma-separated string
    Example: --emails "john@example.com,jane@example.com"
    
    Default: none (no email filtering)
```

### Behavior

1. **Discovery**: Locate all PST files in INPUT (single file or directory scan)
2. **Sorting**: Process PST files in alphabetical order of filename (deterministic)
3. **Processing**: For each PST file, iterate through all folders and messages
4. **Numbering**: Assign global sequence number to each message (00001, 00002, etc.)
5. **Duplicate Detection**: 
   - Check Message-ID header
   - If no Message-ID, generate content hash (SHA-256 of Subject+Date+From+Body)
   - First occurrence → export to OUTPUT_DIR/{seq_num}/
   - Duplicates → export to OUTPUT_DIR/duplicates/{seq_num}/
6. **Export**: Create message.html (always) + optional files based on flags
7. **Progress**: Display "Processing message N/total..." to stderr (unless --quiet)
8. **Summary**: Display statistics after completion (unless --quiet)

### Output Structure

```
<OUTPUT_DIR>/
├── 00001/
│   ├── message.html      # HTML representation of message (always)
│   ├── metadata.txt      # Metadata (if --metadata)
│   ├── headers.txt       # Transport headers (if --headers)
│   ├── attachment1.docx  # Attachments (if --attachments)
│   └── attachment2.pdf
├── 00002/
│   └── message.html
├── duplicates/           # Duplicate messages (if any found)
│   └── 00010/
│       └── message.html
└── emails.csv            # CSV summary (if --csv)
```

### Progress Output (stderr)

```
Processing message 1/5000...
Processing message 2/5000...
...
Processing message 5000/5000...

Export Summary:
  Total messages processed: 5000
  Duplicates found: 127
  Errors encountered: 3
  Messages with keywords: 45  # (if --keywords used)
  Messages with email matches: 89  # (if --emails used)
  Elapsed time: 124.56s
```

With `--quiet` flag, only errors printed to stderr, no progress or summary.

### Exit Codes

```
0   Success (all messages processed, some may have errors but export
    completed)
1   Invalid arguments (missing required flags, invalid paths, etc.)
2   Input not found or not readable
3   Output directory not writable
4   No PST files found in input
64  Fatal error during processing (unrecoverable)
```

### Examples

```bash
# Basic export of single PST file
pst-cli export archive.pst -o ./output

# Export directory of PST files with all options
pst-cli export ./pst_files/ -o ./export \
  --metadata --attachments --headers --csv \
  --keywords "confidential,merger" \
  --emails "ceo@company.com,legal@company.com"

# Quiet mode for scripting (no progress output)
pst-cli export archive.pst -o ./output --quiet

# CSV-only export for quick analysis
pst-cli export ./pst_files/ -o ./output --csv
```

---

## Command: list

Display PST folder structure and message counts without exporting.

### Synopsis

```bash
pst-cli list <PST_FILE>
```

### Arguments

```
<PST_FILE>
    Path to a single PST file to list.
    
    Required: yes
    Type: path (file)
    Validation:
      - Must exist and be readable
      - Must have .pst extension
      - Must be valid PST file format
```

### Output Format

Display folder hierarchy with message counts to stdout:

```
PST: archive.pst (1.2 GB, 4,523 messages)

Folder Structure:
├── Inbox (1,234 messages)
│   ├── Important (45 messages)
│   └── Archive (567 messages)
├── Sent Items (892 messages)
├── Deleted Items (123 messages)
└── Drafts (8 messages)

Total: 4,523 messages across 7 folders
```

### Behavior

1. Open PST file (read-only, minimal loading)
2. Traverse folder hierarchy
3. Count messages per folder (no message content loaded)
4. Display tree structure with counts
5. Display summary totals

### Exit Codes

```
0   Success (PST file listed successfully)
1   Invalid arguments (missing PST file path, etc.)
2   PST file not found or not readable
3   PST file corrupted or invalid format
```

### Examples

```bash
# List single PST file structure
pst-cli list archive.pst

# Pipe output to file for documentation
pst-cli list archive.pst > pst_structure.txt
```

---

## Error Handling

### Error Output Format (stderr)

```
Error: <error_type>: <error_message>
  Context: <additional context>
  Suggestion: <recommended action>
```

Example:
```
Error: Invalid PST file: archive.pst
  Context: File header does not match PST format signature
  Suggestion: Verify file is a valid Outlook PST file
```

### Partial Export Errors

When individual message export fails:

1. Create message folder with sequence number
2. Export partial content to message.html (best-effort)
3. Create error.txt with error details
4. Write error to stderr (continues processing other messages)
5. Increment `errors_encountered` in summary statistics

### Error Types

- **InvalidArgument**: Missing or invalid CLI arguments
- **FileNotFound**: Input file or directory doesn't exist
- **PermissionDenied**: Cannot read input or write to output
- **InvalidPstFormat**: PST file corrupted or not valid format
- **ParseError**: Failed to parse specific message (non-fatal)
- **IoError**: Disk I/O failure during export
- **OutOfMemory**: Memory exhaustion (should not occur with streaming)

---

## Output File Formats

### message.html

HTML representation of email message:

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>[Subject]</title>
  <style>
    /* Basic email styling */
    body { font-family: Arial, sans-serif; max-width: 800px; margin: 20px; }
    .header { border-bottom: 1px solid #ccc; padding-bottom: 10px; margin-bottom: 20px; }
    .header-field { margin: 5px 0; }
    .body { white-space: pre-wrap; }
  </style>
</head>
<body>
  <div class="header">
    <div class="header-field"><strong>From:</strong> John Doe &lt;john@example.com&gt;</div>
    <div class="header-field"><strong>To:</strong> jane@example.com</div>
    <div class="header-field"><strong>Date:</strong> 2023-05-15 14:30:45 UTC</div>
    <div class="header-field"><strong>Subject:</strong> Project Update</div>
  </div>
  <div class="body">
    [Message body content - HTML, RTF-converted, or wrapped plain text]
  </div>
  <!-- If export error occurred: -->
  <div class="error">
    <strong>Export Error:</strong> [error details]
  </div>
</body>
</html>
```

### metadata.txt

Plain text key-value format:

```
Subject: Project Update
From: John Doe <john@example.com>
Date: 2023-05-15T14:30:45Z
To: jane@example.com, bob@example.com
CC: team@example.com
BCC: 
MessageId: <abc123@mail.example.com>
Folder: Inbox/Projects/2023
Size: 45678 bytes
Attachments: report.pdf (1.2 MB), spreadsheet.xlsx (543 KB)
Flags: [Read, Flagged]
Keywords: confidential, project  # (if --keywords used and matched)
Participant Matches: jane@example.com  # (if --emails used and matched)
```

### headers.txt

Raw email headers (preserved formatting):

```
Received: from mail.example.com ...
Return-Path: <john@example.com>
Date: Mon, 15 May 2023 14:30:45 +0000
From: John Doe <john@example.com>
To: Jane Smith <jane@example.com>
Subject: Project Update
Message-ID: <abc123@mail.example.com>
...
```

### emails.csv

CSV format with header row:

```csv
SequenceNumber,MessageID,Subject,From,To,CC,BCC,Date,FolderPath,SizeBytes,AttachmentCount,AttachmentNames,IsDuplicate,KeywordCount,EmailMatchCount
1,<abc123@mail.example.com>,"Project Update","John Doe <john@example.com>","jane@example.com, bob@example.com","team@example.com","",2023-05-15T14:30:45Z,Inbox/Projects/2023,45678,2,"report.pdf, spreadsheet.xlsx",false,2,1
2,<def456@mail.example.com>,"Re: Project Update","jane@example.com","john@example.com","","",2023-05-15T15:45:23Z,Sent Items,12345,0,"",false,0,0
```

**CSV Escaping Rules**:
- Fields containing commas, quotes, or newlines are double-quoted
- Double quotes within fields are escaped as ""
- Newlines within quoted fields are preserved
- Use UTF-8 encoding

---

## Versioning & Compatibility

**Version**: 0.1.0 (initial release)

**Compatibility Promise**:
- CLI arguments and flags remain stable within minor versions (0.x.y)
- Output file formats remain compatible within minor versions
- Breaking CLI changes increment major version

**Deprecation Policy**:
- Deprecated flags show warning but continue to work for one major version
- New flags added in minor versions, default to behavior preserving compatibility

---

## Testing Contract

### Unit Test Coverage

Each CLI command MUST have unit tests verifying:
- Argument parsing (valid and invalid combinations)
- Help text generation
- Exit code correctness

### Integration Test Coverage

Export command MUST be tested with:
- Single PST file with known message count
- Directory with multiple PST files
- PST with various body formats (HTML, RTF, plain text)
- PST with duplicate messages (verify duplicates/ directory)
- PST with corrupted messages (verify partial export + error.txt)
- All flag combinations (--metadata, --attachments, --headers, --csv)
- Keyword and email filtering (verify matching logic)
- Progress output validation (with and without --quiet)

List command MUST be tested with:
- PST with nested folder hierarchy
- PST with empty folders
- Corrupted PST (verify error handling)

### Example Tests

Canonical examples in `examples/` MUST demonstrate:
- `basic_export.rs`: Simple export of sample PST file
- `batch_export.rs`: Directory processing with progress output

---

## Summary

This CLI contract ensures:
✅ Clear, predictable interface following Rust CLI conventions  
✅ Comprehensive error handling with actionable messages  
✅ Deterministic behavior (reproducible results)  
✅ Observable progress and statistics  
✅ Flexible output formats (HTML, CSV, metadata, attachments)  
✅ Scriptable (exit codes, --quiet flag, clean stdout/stderr separation)  

**Implementation Notes**:
- Use `clap` 4.x derive macros for type-safe argument parsing
- Validate all arguments before starting export (fail fast)
- Use structured error types (not string errors)
- Write progress to stderr, data to stdout (pipe-friendly)

**Next**: Generate quickstart guide in [quickstart.md](../quickstart.md)
