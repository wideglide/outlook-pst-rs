# pst-cli

Command-line tool for exporting PST (Personal Storage Table) files to HTML for eDiscovery workflows.

## Features

- **Export PST to HTML**: Convert email messages to numbered HTML files with human-readable content
- **Batch Processing**: Process multiple PST files in a directory with deterministic ordering
- **Duplicate Detection**: Identify and separate duplicate messages based on Message-ID or content hash
- **Metadata Export**: Extract comprehensive metadata (Subject, From, To, Date, etc.) to metadata.txt
- **Attachment Preservation**: Save email attachments with filesystem-safe filenames
- **CSV Summaries**: Generate spreadsheet-ready summaries for analysis and reporting
- **Keyword Filtering**: Search for specific keywords in subject and body (case-insensitive)
- **Participant Filtering**: Find messages involving specific email addresses
- **PST Preview**: List folder structure and message counts without full export

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/microsoft/outlook-pst-rs
cd outlook-pst-rs

# Build pst-cli
cargo build --release -p pst-cli

# Binary will be at: target/release/pst-cli
# Optionally install to cargo bin directory
cargo install --path crates/pst-cli
```

## Quick Start

### Basic Export

Export all messages from a single PST file:

```bash
pst-cli export archive.pst --output ./my_export
```

### Full Export with Metadata

Export with all available information:

```bash
pst-cli export archive.pst --output ./full_export \
  --metadata \
  --attachments \
  --headers \
  --csv
```

### Batch Processing

Process all PST files in a directory:

```bash
pst-cli export ./pst_directory/ --output ./batch_export --csv
```

### List PST Contents

Preview PST structure before exporting:

```bash
pst-cli list archive.pst
```

## Usage

### Export Command

```bash
pst-cli export <INPUT> --output <OUTPUT_DIR> [OPTIONS]
```

**Arguments:**
- `<INPUT>`: Path to PST file or directory containing PST files
- `--output <DIR>`: Output directory for exported messages

**Options:**
- `--metadata`: Export metadata.txt with Subject, From, To, Date, etc.
- `--attachments`: Save email attachments
- `--headers`: Export headers.txt with full transport headers
- `--csv`: Generate emails.csv summary spreadsheet
- `--keywords <LIST>`: Comma-separated keywords to search for (case-insensitive)
- `--emails <LIST>`: Comma-separated email addresses to search for (case-insensitive)
- `--quiet`: Suppress progress indicators and summary statistics
- `--help`: Display help information
- `--version`: Display version information

### List Command

```bash
pst-cli list <PST_FILE>
```

Display PST folder structure and message counts.

## Output Structure

```
output_dir/
├── 00001/
│   ├── message.html      # Message content
│   ├── metadata.txt      # (if --metadata)
│   ├── headers.txt       # (if --headers)
│   └── attachment.pdf    # (if --attachments)
├── 00002/
│   └── message.html
├── duplicates/
│   └── 00005/
│       └── message.html  # Duplicate message
└── emails.csv            # (if --csv)
```

## Requirements

- Rust 1.82 or later
- PST files from Microsoft Outlook

## Performance

- Exports 10,000 messages in under 10 minutes on standard hardware
- Streams large PST files (1GB+) without excessive memory usage

## eDiscovery Compliance

- Deterministic output: Same inputs produce identical numbering
- Accurate content preservation: HTML rendering preserves message content
- Error resilience: Corrupted messages don't stop export; errors logged
- Audit trail: Summary statistics and error logs for defensibility

## Documentation

- [Specification](../../specs/001-pst-cli-tool/spec.md)
- [Quickstart Guide](../../specs/001-pst-cli-tool/quickstart.md)
- [CLI Contract](../../specs/001-pst-cli-tool/contracts/cli-interface.md)

## Contributing

See the repository [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Security

For security concerns, please see [SECURITY.md](../../SECURITY.md).
