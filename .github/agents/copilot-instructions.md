# spec-outlook-pst-rs Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-05

## Active Technologies
- Rust 1.82 (edition 2021) + `outlook-pst` (workspace crate), `clap` 4.5 (CLI parsing), existing `pst-cli` export modules (`export/mod.rs`, `export/exporter.rs`, `export/metadata.rs`) (001-conversation-export-grouping)
- Filesystem output tree under user-provided `--output` directory (001-conversation-export-grouping)
- Rust 1.82 (edition 2021) + `outlook-pst` workspace crate, `compressed-rtf`, `clap` 4.5, `chrono`, `encoding_rs`, `html-escape`, planned parse-based HTML processor (`lol_html`) (001-html-search-inline-export)
- Filesystem export tree under the user-provided `--output` directory (001-html-search-inline-export)

- Rust 1.82 (edition 2021) + pst crate (workspace), compressed-rtf crate (workspace), clap 4.x (CLI parsing with derive), anyhow (application error handling per M-APP-ERROR) (001-pst-cli-tool)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.82 (edition 2021): Follow standard conventions

## Recent Changes
- 003-html-search-inline-export: Added Rust 1.82 (edition 2021) + `outlook-pst` workspace crate, `compressed-rtf`, `clap` 4.5, `chrono`, `encoding_rs`, `html-escape`, planned parse-based HTML processor (`lol_html`)
- 002-conversation-export-grouping: Added Rust 1.82 (edition 2021) + `outlook-pst` (workspace crate), `clap` 4.5 (CLI parsing), existing `pst-cli` export modules (`export/mod.rs`, `export/exporter.rs`, `export/metadata.rs`)

- 001-pst-cli-tool: Added Rust 1.82 (edition 2021) + pst crate (workspace), compressed-rtf crate (workspace), clap 4.x (CLI parsing with derive), anyhow (application error handling per M-APP-ERROR)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
