# outlook-pst-cli

CLI utilities for inspecting Outlook PST files using the `outlook-pst` library.

## Highlights

- Accepts a single .pst file or a directory. When a directory is provided, only `.pst` files are processed (case-insensitive).
- Deterministic ordering for reproducible output:
  - PST files are processed in lexicographic order.
  - Folders are processed by display name then ID; messages are processed by row ID.
- Global zero-based index across all inputs in a single run; the dump command writes per-index folders like `00000/`.

## Install / Build

Run via Cargo from the workspace without installing a separate binary:
- List: `cargo run -p outlook-pst-cli -- list <INPUT>`
- Dump: `cargo run -p outlook-pst-cli -- dump <INPUT> --out-dir <OUT_DIR>`

## Usage

### list
List message metadata to the console.

Flags:
- `--show-headers` — include selected transport headers when present (Message-Id, In-Reply-To, References, X-Mailer, X-Originating-IP, and a few others).
- `--show-attachments` — list basic info for binary attachments.
- `--show-body-types` — indicate which body types exist (text, html, rtf).
- `--responsive-emails email1,email2` — mark as responsive if any participant matches.
- `--keywords word1,word2` — report which keywords are found in the body (best-effort plain-text extraction).
- `--csv` — also write `emails.csv` to the current working directory.

Examples:
- `outlook-pst-cli list <path/to/file.pst>`
- `outlook-pst-cli list <path/to/dir> --show-headers --show-attachments --show-body-types --csv`

### dump
Write each email to HTML with a stable index and optional attachments.

Required:
- `--out-dir <OUT_DIR>` — output directory. Each message is written to `<OUT_DIR>/<INDEX>/message.html` where `<INDEX>` is zero-based and padded to 5 digits (e.g., `00012`).

Optional:
- `--attachments` — save attachments in the same folder as `message.html`. Inline content-id attachments are saved using their content-id when available; other binary attachments are saved by filename.
- `--metadata` — write message metadata (Subject, From, Date, To, Cc, Bcc, MessageId, Folder, Size, Flags, Responsive Emails, Keywords, Attachments) to `metadata.txt` in each message folder.
- `--headers` — write transport headers (Message-ID, In-Reply-To, References, X-Mailer, X-Originating-IP, Return-Path, Received chain, etc.) to `headers.txt` in each message folder when available.
- `--responsive-emails email1,email2` — mark as responsive if any participant matches.
- `--keywords word1,word2` — report which keywords appear in the body.
- `--csv` — also write `emails.csv` to `<OUT_DIR>`.

Examples:
- `outlook-pst-cli dump <path/to/file.pst> --out-dir out --attachments --csv`
- `outlook-pst-cli dump <path/to/dir> --out-dir out --metadata --headers`
- `outlook-pst-cli dump <path/to/file.pst> --out-dir out --attachments --metadata --headers --csv`

## CSV output

When `--csv` is specified, a CSV summary of all processed emails is produced with columns:

`index, subject, date, from, to, cc, size, number-of-responsive-emails, number-of-keywords, number-of-attachments, MessageId, pst-store-name, duplicate`

- For `list`, the CSV is written to the current working directory as `emails.csv`.
- For `dump`, the CSV is written to the specified output directory as `emails.csv`.
- The `duplicate` column indicates whether a message has a duplicate Message-ID that was already processed.

## Output details (dump)

- **HTML body**: The HTML file includes only core headers in a metadata table: Subject, From, Date, To, Cc, Bcc, and Attachments (if present).
- **Body selection order**: HTML body (if present) → plain text body (wrapped in `<pre>`) → decoded RTF (wrapped in `<pre>`).
- **metadata.txt** (when `--metadata` is used): Contains all message metadata including Subject, From, Date, To, Cc, Bcc, MessageId, Folder, Size, Flags, Responsive Emails, Keywords, and Attachments.
- **headers.txt** (when `--headers` is used): Contains transport headers like Message-ID, In-Reply-To, References, X-Mailer, X-Originating-IP, Return-Path, Received chain, and other X-headers. Only written if transport headers exist for the message.
- **Attachments** (when `--attachments` is used): Saved in the same folder as `message.html`.
- **Duplicate detection**: Messages with duplicate Message-IDs are written to `<OUT_DIR>/duplicates/<INDEX>/` instead of the main output directory.

## Processing summary

At the end of a run, a summary is printed with totals for folders, messages, size, messages with attachments, and any messages skipped due to errors.

## Behavior and limits

- Unicode PST files are supported. ANSI PST support is not implemented.
- Keyword matching is a simple case-insensitive substring search on a best-effort plain body.
- Participant matching for `--responsive-emails` is best-effort and based on extracting addresses from headers and recipient tables.
- Deterministic traversal means repeated runs over the same inputs produce identical index assignments.

## Examples (from this repo)

- List a sample PST:
  - `cargo run -p outlook-pst-cli -- list crates/pst/Example-001.pst --show-headers --show-attachments --show-body-types`
- Dump a sample PST with attachments and CSV:
  - `cargo run -p outlook-pst-cli -- dump crates/pst/Example-001.pst --out-dir target/email_html_out --attachments --csv`
- Dump with metadata and transport headers:
  - `cargo run -p outlook-pst-cli -- dump crates/pst/Example-001.pst --out-dir target/email_html_out --metadata --headers --attachments --csv`
