# Research: HTML Search and Inline Export

**Phase**: 0 - Research & Design Decisions  
**Date**: 2026-03-09  
**Feature**: [spec.md](spec.md) | [plan.md](plan.md)

## Overview

This document records the design choices for making HTML keyword matching user-visible-text aware and for rewriting inline attachment references in exported `message.html` files.

## HTML Keyword Matching Strategy

### Decision
Use a parse-based HTML text extraction step before keyword matching so `KeywordMatcher` evaluates only visible text for HTML bodies.

### Rationale
- Current keyword matching lowercases the raw body string and performs substring checks, which causes false positives on markup and hidden HTML content.
- HTML-visible-text extraction addresses the user-facing problem without changing existing subject matching or non-HTML body behavior.
- The body HTML is already loaded into memory during export staging, so the extraction can happen without additional PST reads.

### Alternatives considered
- Continue searching raw HTML: rejected because it violates the feature requirements.
- Regex-strip tags before searching: rejected because it is brittle for malformed HTML and does not safely exclude script/style/comment content.
- Switch keyword matching to tokenized or whole-word semantics: rejected because the feature only changes HTML interpretation, not the underlying substring matching contract.

## HTML Processing Dependency Choice

### Decision
Adopt `lol_html` as the single parse-based HTML processing dependency for both visible-text extraction and inline-reference rewriting.

### Rationale
- The user explicitly asked for a parse-based rewriter using `lol_html` or `html5ever` rather than regex replacement.
- `lol_html` provides streaming, selector-driven rewriting without requiring a full DOM, keeping the implementation smaller and more localized.
- Using one HTML processor for both matching and rewriting minimizes dependency surface and reduces duplicated parsing logic.

### Alternatives considered
- `html5ever` with a DOM tree: rejected because it would require more scaffolding and state management for a feature meant to stay lightweight.
- Separate crates for extraction and rewriting: rejected because it increases complexity without clear benefit.
- Custom HTML scanner: rejected because it would be hard to make correct and maintainable.

## Visible-Text Scope

### Decision
Treat the following as searchable in HTML bodies: normal text nodes rendered in the document body after parsing. Exclude text appearing only in tags, comments, `<script>`, and `<style>` blocks.

### Rationale
- This matches the accepted feature scope and aligns search results with what a human reviewer reads in the rendered message.
- Excluding comments and hidden code/text blocks prevents common false positives in email markup.

### Alternatives considered
- Exclude additional hidden content such as CSS-hidden elements: rejected for Phase 1 because it requires layout/CSS interpretation outside the requested scope.
- Search attribute values such as `alt` or `title`: rejected because the spec calls for visible content rather than markup metadata.

## Attachment Metadata Capture

### Decision
Extend the `pst-cli` attachment model to capture optional `PidTagAttachContentId` and `PidTagAttachContentLocation` alongside the existing filename, bytes, and content type during `extract_attachments`.

### Rationale
- `extract_attachments` already opens each attachment sub-node and reads raw properties, so the additional metadata can be captured at the same point with minimal extra cost.
- The low-level PST crate already exposes `AttachmentProperties::get`, so this feature can likely avoid broadening the `outlook-pst` public API.
- Carrying the metadata in the staged `Attachment` record keeps matching local to the export flow.

### Alternatives considered
- Re-open attachment properties during HTML writing: rejected because it adds redundant work and complicates the exporter.
- Add a large new attachment abstraction in `outlook-pst`: rejected because the current need is narrow and CLI-specific.

## Attachment Filename Planning

### Decision
Centralize attachment filename sanitization and collision handling into a reusable attachment export plan computed before `message.html` is written, and reuse that plan for both file writes and HTML rewriting.

### Rationale
- Current collision handling exists only inside `write_attachments`, which is too late for `message.html` rewriting to know the final filenames.
- A shared plan guarantees `message.html` references and exported files stay in sync under filename sanitization and duplicate-name suffixing.
- This design avoids post-write scanning or speculative filename guessing.

### Alternatives considered
- Reimplement filename collision logic in the HTML rewriter: rejected because duplicated logic would drift and produce mismatches.
- Write attachments first and then scan the filesystem to discover filenames: rejected as unnecessary IO and complexity.

## Inline Reference Rewrite Scope

### Decision
Rewrite URL-valued HTML attributes used for inline resources in exported message bodies, starting with `src` and `href`, when the value is a resolvable `cid:` or content-location reference for an attachment exported with the same message.

### Rationale
- `src` covers the dominant inline-image case and `href` covers linked inline attachments.
- Limiting the first implementation to standard URL attributes keeps the behavior predictable and avoids overreaching into CSS parsing or script content.
- References that cannot be matched remain unchanged, preserving message fidelity.

### Alternatives considered
- Rewrite every possible attribute or inline CSS `url(...)`: rejected as broader than the feature requires.
- Rewrite only `cid:` and ignore content-location: rejected because the spec explicitly requires both.

## Ambiguous Attachment Match Handling

### Decision
Treat any inline reference that matches more than one candidate attachment as unresolved and leave the original HTML reference unchanged.

### Rationale
- The specification requires rewriting only references that can be matched unambiguously.
- Preserving the original reference is safer than guessing and linking to the wrong exported file.
- This keeps ambiguous cases aligned with the unresolved-reference behavior already defined elsewhere.

## Test Strategy

### Decision
Cover the feature with a mix of unit and integration tests:
- Unit tests for HTML-visible-text extraction, reference normalization, and filename planning.
- Exporter-focused tests for rewriting resolvable and unresolvable references.
- Integration tests using the existing `pst-cli` export flow for keyword filtering and attachment-export output.

### Rationale
- Most of the new logic is deterministic string/property transformation and is best validated in fast unit tests.
- Integration coverage confirms the feature works inside the staged export pipeline and preserves existing output structure.
- This satisfies the constitution requirement for regression tests on bug-fix behavior.

### Alternatives considered
- Rely only on integration tests with PST fixtures: rejected because crafting exact HTML edge cases is slower and more brittle.
- Rely only on unit tests: rejected because the export pipeline integration matters for correctness.

## Resolved Clarifications

All technical unknowns for this feature are resolved. No `NEEDS CLARIFICATION` items remain.