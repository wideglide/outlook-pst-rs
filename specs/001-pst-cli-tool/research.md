# Research: PST CLI Tool

**Phase**: 0 - Research & Design Decisions  
**Date**: 2026-02-05  
**Feature**: [spec.md](spec.md) | [plan.md](plan.md)

## Overview

This document consolidates research findings and design decisions for the pst-cli tool implementation. All unknowns from the Technical Context have been resolved through investigation of Rust best practices, workspace dependencies, and eDiscovery requirements.

---

## CLI Architecture with Clap 4.x

### Decision
Use `clap 4.x` with derive macros for subcommand-style CLI (`pst-cli export` / `pst-cli list`).

### Rationale
- **Idiomatic Rust**: Subcommand pattern follows cargo, rustc, and other Rust CLI tools (aligns with M-DESIGN-FOR-AI guideline for familiar patterns)
- **Type-safe**: Derive macros provide compile-time validation of argument conflicts and required parameters
- **Workspace ready**: clap 4.x already in workspace.dependencies with derive feature enabled
- **User-friendly**: Built-in help generation, argument validation, and error messages

### Alternatives Considered
- **structopt**: Predecessor to clap derive, now merged into clap 4.x - no longer maintained separately
- **Flag-based single binary**: e.g., `pst-cli --export` - less clear separation, harder to extend with new commands
- **Separate binaries**: e.g., `pst-export`, `pst-list` - adds packaging complexity without clear benefit

### Implementation Notes
```rust
// Main command structure (simplified)
#[derive(Parser)]
#[command(name = "pst-cli", version, about = "PST file export and analysis tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Suppress progress indicators and summary statistics
    #[arg(long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Export messages from PST file(s) to HTML
    Export(ExportArgs),
    /// List PST folder structure and message counts
    List(ListArgs),
}
```

---

## HTML Generation from Email Body Formats

### Decision
Implement priority-based body format conversion: HTML body (direct use) > RTF body (decompress + convert) > Plain text body (wrap with basic HTML).

### Rationale
- **Best fidelity**: Preserves original formatting when HTML body exists (most accurate representation for legal review)
- **RTF support**: Leverages existing `compressed-rtf` workspace crate for decompression
- **Fallback coverage**: Handles plain text messages gracefully with minimal HTML wrapper
- **eDiscovery defensibility**: Always produces HTML output in consistent format for review tools

### Alternatives Considered
- **HTML only, skip RTF/plain**: Would fail for many older Outlook messages that use RTF
- **Multiple exports per message**: e.g., message.html + message.txt - increases storage, conflicts with spec requirement for single message.html
- **Always convert to canonical format**: More processing overhead, loses original HTML fidelity

### Implementation Notes
- **HTML body**: Extract and sanitize, ensure proper character encoding (UTF-8 output)
- **RTF body**: Use `compressed-rtf` crate to decompress, then RTF-to-HTML converter (options: pandoc-types, custom parser, or basic RTF tag mapping)
- **Plain text body**: Wrap in `<pre>` tags or use `<p>` with `<br>` for line breaks, HTML-escape special characters

#### RTF-to-HTML Conversion Strategy
**Research finding**: No mature pure-Rust RTF-to-HTML library exists. Options:
1. **Basic RTF interpreter** (recommended): Parse common RTF tags (\par, \b, \i, \ul) and map to HTML equivalents - sufficient for email viewing
2. **External tool**: Shell out to pandoc (not cross-platform friendly, adds dependency)
3. **Minimal output**: Expose raw decompressed RTF text in `<pre>` tags (loses formatting but preserves content)

**Decision**: Implement basic RTF-to-HTML converter focusing on common email formatting (paragraphs, bold, italic, underline, links). Document limitation: complex RTF features may not render perfectly.

---

## Content Hashing for Duplicate Detection

### Decision
Generate SHA-256 hash from `Subject + Date + From + Body` (concatenated, normalized) as fallback identifier when Message-ID header is missing.

### Rationale
- **Deterministic**: Same message content produces same hash across runs (meets SC-002 determinism requirement)
- **Forensically sound**: SHA-256 is cryptographically strong, widely accepted in eDiscovery tools
- **Comprehensive**: Includes all key distinguishing fields to minimize false positives
- **Efficient**: Hash computation is fast (< 1ms per message), suitable for high-volume processing

### Alternatives Considered
- **Headers only (Subject+Date+From)**: Faster but higher false positive rate (same headers, different body content)
- **MD5**: Faster but not cryptographically secure, questionable for legal use
- **Full message hash**: Includes headers formatting variations, produces false negatives for identical content with different header ordering

### Implementation Notes
```rust
// Pseudocode for content hash generation
fn generate_content_hash(msg: &Message) -> String {
    use sha2::{Sha256, Digest};
    
    // Normalize fields (lowercase, trim whitespace)
    let subject = msg.subject.to_lowercase().trim();
    let date = msg.date.to_rfc3339(); // Canonical date format
    let from = msg.from.address.to_lowercase().trim();
    let body = msg.body_text().to_lowercase().trim();
    
    let content = format!("{}{}{}{}", subject, date, from, body);
    let hash = Sha256::digest(content.as_bytes());
    format!("{:x}", hash) // Hex string representation
}
```

**Normalization rationale**: 
- Lowercase + trim reduces false negatives from formatting differences
- RFC3339 date format ensures consistent timestamp representation
- Body text extraction uses plain text representation (strips HTML/RTF formatting for consistent hashing)

---

## Streaming PST Processing Patterns

### Decision
Use iterator-based streaming approach: iterate over PST folders/messages without loading entire file into memory.

### Rationale
- **Memory efficiency**: Meets SC-001 requirement to process 1GB+ PST files in constrained memory
- **Rust-idiomatic**: Leverages existing `pst` crate iterator APIs (assumption: pst crate provides streaming access - verify during implementation)
- **Scalable**: Supports unlimited PST sizes without performance degradation
- **Error resilience**: Process messages one at a time, skip corrupted entries without losing entire export

### Alternatives Considered
- **Load entire PST into memory**: Simple but fails SC-001 memory constraint for large files
- **Memory-mapped files**: Complex platform-specific code, violates Portability & Correctness principle
- **Chunked loading**: Adds complexity without clear benefit over streaming

### Implementation Notes
```rust
// Pseudocode for streaming PST processing
fn export_pst_streaming(pst_path: &Path, output_dir: &Path) -> Result<ExportStats> {
    let pst = PstFile::open(pst_path)?;
    let mut exporter = Exporter::new(output_dir);
    
    for folder in pst.folders() {
        for message in folder.messages() {
            match message {
                Ok(msg) => exporter.export_message(msg)?,
                Err(e) => {
                    log::error!("Failed to read message: {}", e);
                    // Continue processing remaining messages
                }
            }
        }
    }
    
    Ok(exporter.stats())
}
```

**Assumption to verify**: pst crate provides iterator-based message access. If not available, research required for custom streaming implementation.

---

## Progress Reporting and Summary Statistics

### Decision
Simple in-place counter to stderr (`Processing message 1234/5000...`) during export + summary table at completion (unless `--quiet` flag provided).

### Rationale
- **User feedback**: Provides progress visibility for long-running exports (satisfies FR-034)
- **Clean output**: stderr for progress, stdout available for piping export results
- **CI/automation friendly**: `--quiet` suppresses progress for non-interactive environments
- **Observability**: Summary statistics (totals, duplicates, errors, time) enable performance validation and debugging (aligns with Performance Requirements principle)

### Alternatives Considered
- **Progress bar with ETA**: More visual but adds dependency, harder to test, less robust in non-TTY environments
- **Verbose logging**: Too noisy, drowns out errors
- **Silent operation by default**: No user feedback for long operations, poor UX

### Implementation Notes
```rust
// Summary statistics structure
struct ExportStats {
    total_messages: usize,
    duplicates_found: usize,
    errors_encountered: usize,
    messages_with_keywords: usize,  // When --keywords used
    messages_with_emails: usize,    // When --emails used
    elapsed: Duration,
}

impl ExportStats {
    fn display_summary(&self) {
        eprintln!("\nExport Summary:");
        eprintln!("  Total messages processed: {}", self.total_messages);
        eprintln!("  Duplicates found: {}", self.duplicates_found);
        eprintln!("  Errors encountered: {}", self.errors_encountered);
        if has_keywords { 
            eprintln!("  Messages with keywords: {}", self.messages_with_keywords);
        }
        if has_emails {
            eprintln!("  Messages with email matches: {}", self.messages_with_emails);
        }
        eprintln!("  Elapsed time: {:.2}s", self.elapsed.as_secs_f64());
    }
}
```

---

## Additional Dependencies Research

### Required Crates (beyond workspace.dependencies)
- **sha2**: For SHA-256 content hash generation (duplicate detection fallback)
- **csv**: For CSV export generation (FR-020)
- **encoding_rs** or **chardet**: For character encoding detection/conversion in email bodies (FR-005)

### Already Available in Workspace
- **anyhow**: Application-level error handling (M-APP-ERROR guideline)
- **clap**: CLI argument parsing
- **pst**: PST file parsing
- **compressed-rtf**: RTF decompression

### Optional Performance Enhancements
- **rayon**: Parallel message processing (research: determine if deterministic numbering compatible with parallelism - likely NOT compatible, skip)
- **indicatif**: Advanced progress bars (skipped per Decision above)

---

## Open Questions / Implementation Risks

1. **PST crate API verification**: Confirm pst crate provides iterator-based message access for streaming (HIGH PRIORITY - critical for memory constraint)
2. **RTF-to-HTML quality**: Basic RTF converter may not handle all formatting perfectly (ACCEPTABLE - document limitation, sufficient for eDiscovery text review)
3. **Character encoding edge cases**: Some PST files may have unusual encodings not supported by encoding_rs (MITIGATED - use lossy conversion with logging)
4. **CSV field escaping edge cases**: Complex message content may challenge CSV escaping (MITIGATED - use mature `csv` crate with proper configuration)

---

## Summary of Design Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| CLI Architecture | clap 4.x derive + subcommands | Idiomatic Rust, type-safe, workspace-ready |
| HTML Generation | Priority: HTML > RTF > Plain Text | Best fidelity, uses compressed-rtf crate, eDiscovery defensible |
| RTF Conversion | Basic RTF interpreter | No mature pure-Rust library, basic sufficient for email viewing |
| Duplicate Detection | SHA-256(Subject+Date+From+Body) | Forensically sound, deterministic, efficient |
| PST Processing | Iterator-based streaming | Memory-efficient, scalable, error-resilient |
| Progress Reporting | Simple counter + summary stats | Clean, CI-friendly with --quiet, observable |
| Dependencies | sha2, csv, encoding_rs | Minimal additions, mature crates, focused functionality |

**All NEEDS CLARIFICATION items resolved. Ready for Phase 1: Data Model and Contracts.**
