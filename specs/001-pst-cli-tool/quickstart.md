# Quickstart: PST CLI Tool

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [data-model.md](data-model.md) | [CLI Contract](contracts/cli-interface.md)  
**Updated**: 2026-02-05

## Overview

This quickstart guide demonstrates common workflows for using `pst-cli` to export and analyze PST files for eDiscovery. Follow these examples to quickly accomplish typical tasks.

---

## Installation

### Prerequisites
- Rust 1.82 or later
- PST files to process

### Build from Source

```bash
# Clone repository (if not already cloned)
git clone https://github.com/microsoft/outlook-pst-rs
cd outlook-pst-rs

# Build pst-cli
cargo build --release --package pst-cli

# Binary will be at: target/release/pst-cli
# (On Windows: target\release\pst-cli.exe)

# Optionally install to cargo bin directory
cargo install --path crates/pst-cli
```

### Verify Installation

```bash
pst-cli --version
# Output: pst-cli 0.1.0

pst-cli --help
# Displays available commands and options
```

---

## Common Workflows

### 1. Basic Export: Single PST File

Export all messages from a PST file to HTML:

```bash
pst-cli export archive.pst --output ./my_export
```

**What happens**:
- Creates `./my_export/` directory
- Exports each message to numbered folders (00001/, 00002/, etc.)
- Each folder contains `message.html` with message content
- Displays progress and summary statistics

**Output structure**:
```
my_export/
├── 00001/
│   └── message.html
├── 00002/
│   └── message.html
└── ...
```

---

### 2. Full Export with Metadata and Attachments

Export messages with all available information:

```bash
pst-cli export archive.pst --output ./full_export \
  --metadata \
  --attachments \
  --headers \
  --csv
```

**What happens**:
- Exports HTML messages (always)
- Creates `metadata.txt` in each message folder (Subject, From, To, etc.)
- Saves attachments with original filenames (sanitized)
- Exports full email headers to `headers.txt`
- Generates `emails.csv` summary at output root

**Output structure**:
```
full_export/
├── 00001/
│   ├── message.html
│   ├── metadata.txt
│   ├── headers.txt
│   ├── document.pdf
│   └── spreadsheet.xlsx
├── 00002/
│   └── ...
└── emails.csv
```

---

### 3. Batch Processing: Multiple PST Files

Process all PST files in a directory:

```bash
pst-cli export ./pst_directory/ --output ./batch_export --csv
```

**What happens**:
- Scans `./pst_directory/` for all `.pst` files
- Processes files in alphabetical order (deterministic)
- Assigns sequential numbering across all files (00001, 00002, ...)
- Generates single unified `emails.csv` covering all messages

**Example**: Directory contains `alice.pst` (100 msgs) and `bob.pst` (50 msgs)
- alice.pst messages → 00001 to 00100
- bob.pst messages → 00101 to 00150

---

### 4. Duplicate Detection

Identify and separate duplicate messages:

```bash
pst-cli export ./pst_files/ --output ./dedup_export
```

**What happens**:
- Detects duplicates using Message-ID header
- For messages without Message-ID: generates content hash (SHA-256)
- First occurrence → exported to main output directory
- Duplicates → exported to `duplicates/` subdirectory
- Summary shows count of duplicates found

**Output structure**:
```
dedup_export/
├── 00001/  # First occurrence
├── 00002/  # Unique message
├── duplicates/
│   ├── 00005/  # Duplicate of 00001
│   └── 00012/  # Duplicate of 00002
└── ...
```

**Tip**: Use `--csv` to see duplicate status in spreadsheet:
```csv
SequenceNumber,MessageID,Subject,IsDuplicate,...
1,<abc@example.com>,"Meeting Notes",false,...
5,<abc@example.com>,"Meeting Notes",true,...
```

---

### 5. Keyword Search

Find messages containing specific keywords (case-insensitive):

```bash
pst-cli export archive.pst --output ./keyword_search \
  --keywords "confidential,merger,acquisition" \
  --metadata \
  --csv
```

**What happens**:
- Searches subject and body for keywords (case-insensitive)
- Reports matched keywords in `metadata.txt` (if --metadata used)
- CSV includes `KeywordCount` column showing matches per message
- Summary shows total messages with keyword matches

**Example metadata.txt**:
```
Subject: Confidential: Merger Planning
...
Keywords: confidential, merger
```

**Example CSV**:
```csv
SequenceNumber,Subject,KeywordCount,...
1,"Confidential: Merger Planning",2,...
2,"Weekly Update",0,...
```

---

### 6. Email Participant Filtering

Find messages involving specific email addresses:

```bash
pst-cli export ./pst_files/ --output ./participant_search \
  --emails "ceo@company.com,legal@company.com,external@vendor.com" \
  --metadata \
  --csv
```

**What happens**:
- Searches From, To, CC, BCC fields for specified addresses
- Matching is case-insensitive on email address only
- Reports matched participants in `metadata.txt`
- CSV includes `EmailMatchCount` column

**Example metadata.txt**:
```
Subject: Contract Review
To: legal@company.com, accounting@company.com
Participant Matches: legal@company.com
```

---

### 7. Combined Filtering

Search for messages with both keywords and specific participants:

```bash
pst-cli export ./litigation_hold/ --output ./discovery_export \
  --keywords "contract,agreement,settlement" \
  --emails "plaintiff@example.com,defendant@example.com" \
  --metadata \
  --attachments \
  --csv
```

**Use case**: eDiscovery for litigation - find all messages discussing contracts involving specific parties, preserve attachments for evidence.

---

### 8. List PST Contents (Preview)

Quickly view PST structure without exporting:

```bash
pst-cli list archive.pst
```

**Output**:
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

**Use case**: Assess PST contents before committing to full export.

---

### 9. Quiet Mode for Scripting

Suppress progress output for automated workflows:

```bash
pst-cli export archive.pst --output ./export --quiet
```

**What happens**:
- No progress indicator shown
- No summary statistics displayed
- Only errors written to stderr
- Exit code indicates success/failure

**Use case**: CI pipelines, cron jobs, batch scripts where progress output is unnecessary.

---

### 10. CSV-Only Quick Analysis

Generate just the CSV summary without exporting HTML/attachments:

```bash
pst-cli export ./pst_files/ --output ./analysis --csv --quiet
```

**What happens**:
- Exports `message.html` files (always created)
- Generates `emails.csv` with metadata
- No progress output (--quiet)
- Ideal for quick data extraction

**Then analyze with spreadsheet tools**:
```bash
# Open CSV in Excel/LibreOffice
open ./analysis/emails.csv

# Or command-line analysis
grep "confidential" ./analysis/emails.csv | wc -l
```

---

## Troubleshooting

### Error: "No PST files found in input"

**Cause**: Directory contains no `.pst` files or path is incorrect.

**Solution**:
```bash
# Verify directory path
ls -la ./pst_directory/*.pst

# Ensure files have .pst extension (case-sensitive on Linux/macOS)
```

---

### Error: "Output directory not writable"

**Cause**: Insufficient permissions for output directory.

**Solution**:
```bash
# Check permissions
ls -ld ./output

# Create directory with proper permissions
mkdir -p ./output
chmod 755 ./output
```

---

### Partial Export Warnings

**Symptom**: Some message folders contain `error.txt` file.

**Cause**: Individual messages failed to export (corrupted, invalid encoding, etc.).

**What to check**:
1. Open `error.txt` in message folder for details
2. Check stderr output for specific error
3. Review `message.html` - partial content may still be exported

**Example error.txt**:
```
Export Error: Failed to decompress RTF body
  Message-ID: <corrupted@example.com>
  PST offset: 0x1A2B3C
  Error: Invalid RTF compression header

Partial content exported to message.html (subject, headers, metadata only).
```

---

### Performance: Slow Export

**Symptoms**: Export takes longer than expected for large PST files.

**Recommendations**:
1. **Check disk I/O**: Slow storage (network drives, USB 2.0) impacts performance
2. **Monitor memory**: Ensure system isn't swapping (htop/Activity Monitor)
3. **Skip attachments initially**: Attachments significantly increase export time
   ```bash
   # Fast export: HTML only
   pst-cli export large.pst -o ./quick_export
   
   # Then add attachments if needed
   pst-cli export large.pst -o ./full_export --attachments
   ```
4. **Use --quiet**: Progress counter has minimal overhead but can be skipped

**Expected performance**: ~10,000 messages in <10 minutes on modern hardware (SSD, 8GB+ RAM).

---

## Best Practices

### eDiscovery Workflows

1. **Always enable --csv**: Provides searchable metadata for review
2. **Enable --metadata for key messages**: Preserves audit trail
3. **Use --attachments judiciously**: Only when attachments needed (saves time/space)
4. **Document keyword/email filters**: Keep record of search criteria for defensibility
5. **Verify duplicate detection**: Check `duplicates/` folder for expected duplicates

### Repeatability

To ensure identical results across multiple runs:
```bash
# Same command always produces same sequence numbers
pst-cli export ./pst_files/ -o ./export_v1
pst-cli export ./pst_files/ -o ./export_v2

# Compare outputs (should be identical)
diff -r ./export_v1 ./export_v2
```

Deterministic factors:
- ✅ PST files processed alphabetically
- ✅ Messages numbered sequentially in processing order
- ✅ Duplicate detection based on stable hash
- ❌ Timestamps in output files (modification times differ)

---

## Next Steps

After completing export:

1. **Review `emails.csv`**: Open in Excel/LibreOffice for analysis
2. **Spot-check message.html files**: Verify content quality in browser
3. **Check duplicates**: Review `duplicates/` folder for expected behavior
4. **Preserve output**: Archive export directory for legal hold compliance

### Example Analysis Workflow

```bash
# 1. Export with all metadata
pst-cli export ./case_files/ -o ./evidence --metadata --attachments --csv

# 2. Open CSV in spreadsheet tool
open ./evidence/emails.csv

# 3. Filter/sort in spreadsheet:
#    - Sort by Date to find relevant timeframe
#    - Filter by Keywords to find key documents
#    - Check IsDuplicate column to identify redundant messages

# 4. Review specific messages in browser
open ./evidence/00042/message.html

# 5. Archive for preservation
tar -czf evidence_export_$(date +%Y%m%d).tar.gz ./evidence/
```

---

## Additional Resources

- **Specification**: [spec.md](spec.md) - Feature requirements and acceptance criteria
- **Data Model**: [data-model.md](data-model.md) - Internal data structures and flow
- **CLI Contract**: [contracts/cli-interface.md](contracts/cli-interface.md) - Complete CLI reference
- **Research**: [research.md](research.md) - Design decisions and alternatives

For bugs or feature requests, see repository issue tracker.
