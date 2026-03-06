# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-02-06

### Added

- **Export command** (`pst-cli export`): Convert PST email messages to numbered HTML files
  - Single PST file and batch directory processing (alphabetical, deterministic)
  - HTML conversion from HTML body, RTF body (via compressed-rtf decompression), or plain text
  - 5-digit zero-padded sequential numbering (00001, 00002, ...)
  - Character encoding handling with UTF-8 conversion
- **Duplicate detection**: Identify duplicates via Message-ID with SHA-256 content hash fallback
  - First occurrence in main output directory, duplicates in `duplicates/` subdirectory
- **Metadata export** (`--metadata`): Subject, From, To, CC, BCC, Date, Message-ID, Folder, Size, Attachments, Flags
  - Recipient extraction from PST recipient table with SMTP address prioritization
  - "Display Name <email>" formatting for all address fields
  - Exchange DN/X500 address filtering
- **Attachment export** (`--attachments`): Save email attachments with filesystem-safe filenames
  - Collision handling with numeric suffixes for duplicate attachment names
- **Headers export** (`--headers`): Full transport headers preserved in headers.txt
- **CSV summary** (`--csv`): Spreadsheet-ready emails.csv with per-message metadata rows
- **Keyword filtering** (`--keywords`): Case-insensitive search in subject and body
  - Comma-separated keyword list, distinct match counting, metadata and CSV reporting
- **Email participant filtering** (`--emails`): Case-insensitive address matching in From/To/CC/BCC
  - Address extraction from "Display Name <addr>" format, metadata and CSV reporting
- **List command** (`pst-cli list`): Display PST folder structure with message counts per folder
- **Progress reporting**: Message counter and summary statistics (unless `--quiet`)
- **Error resilience**: Corrupted messages logged to stderr without stopping export
