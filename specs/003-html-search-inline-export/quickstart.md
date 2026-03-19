# Quickstart: HTML Search and Inline Export

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [data-model.md](data-model.md) | [contracts/cli-interface.md](contracts/cli-interface.md)  
**Updated**: 2026-03-09

## Overview

This feature keeps the existing `pst-cli export` command shape while improving two behaviors:
- HTML keyword filtering ignores text that appears only in tags, scripts, styles, and comments.
- Exported `message.html` rewrites resolvable inline attachment references to the attachment files written for the same message.

## Prerequisites

- Build `pst-cli`:

```bash
cargo build --release -p pst-cli
```

- Have a PST file or directory of PST files available.

## Run Export with HTML-Sensitive Keyword Filtering

```bash
./target/release/pst-cli export ./test/example-001.pst \
  --output ./out-keywords \
  --metadata \
  --keywords invoice,confidential
```

Expected behavior:
- Messages with `invoice` or `confidential` in visible HTML body text or subject are reported as matches.
- Messages where those strings appear only inside HTML tags, comments, `<script>`, or `<style>` blocks are not treated as matches.
- `metadata.txt` still reports `Keywords:` using the existing output format.

## Run Export with Attachments and Rewritten Inline HTML

```bash
./target/release/pst-cli export ./test/example-001.pst \
  --output ./out-inline \
  --attachments \
  --metadata
```

Expected behavior:
- `message.html` is written as usual.
- Attachment files are written into the same message directory using the normal sanitization and collision rules.
- Resolvable `cid:` and content-location references in `message.html` point to those local attachment files.
- External URLs and unresolved references remain unchanged.

## Combined Validation Flow

```bash
./target/release/pst-cli export ./test \
  --output ./out-full \
  --keywords invoice,logo \
  --attachments \
  --metadata \
  --csv
```

Expected behavior:
- Keyword filtering works for both subject/plain text and visible HTML text.
- Attachment files and rewritten `message.html` output remain in sync.
- Existing CSV and metadata outputs continue to work without new flags.

## Suggested Test Commands

```bash
cargo test -p pst-cli keyword_test html_test attachment_test filtering_test export_test
```

## Quick Validation Checklist

1. Confirm a keyword appearing only in raw HTML markup does not produce a `Keywords:` hit.
2. Confirm a keyword appearing in visible HTML text still produces a hit.
3. Confirm exported `message.html` loads inline image or attachment references from the local message directory when the attachment exists.
4. Confirm unresolved inline references and external links remain unchanged.

## Benchmark Results

Run benchmarks with: `cargo test -p pst-cli --test bench -- html_export --nocapture`

| Operation | Size | Time/op | Throughput |
|-----------|------|---------|------------|
| visible_text_extraction | 10 paragraphs (1.1 KB) | ~82 µs | ~12,300 ops/sec |
| visible_text_extraction | 200 paragraphs (20 KB) | ~855 µs | ~1,170 ops/sec |
| inline_rewrite | 10 paragraphs, 5 images (1.3 KB) | ~81 µs | ~12,300 ops/sec |
| inline_rewrite | 200 paragraphs, 50 images (22 KB) | ~897 µs | ~1,100 ops/sec |

At ~1,000 ops/sec for large messages, both operations add negligible overhead
to a pipeline that already performs disk I/O per message.

## Example

A runnable example demonstrating all three user stories:

```bash
cargo run -p pst-cli --example html_search_inline_export
```
5. Confirm attachment filenames referenced in `message.html` match the files written to disk.