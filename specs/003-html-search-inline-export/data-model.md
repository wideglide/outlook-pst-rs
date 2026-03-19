# Data Model: HTML Search and Inline Export

**Feature**: [spec.md](spec.md) | [plan.md](plan.md) | [research.md](research.md)  
**Phase**: 1 - Design  
**Date**: 2026-03-09

## Overview

This model extends the current `pst-cli` export pipeline so HTML bodies can be searched using visible text and exported attachments can be matched back to inline HTML references deterministically.

## Entities

### ExportMessageRecord

Represents one staged message in the `pst-cli` export pipeline after PST extraction and before filesystem writes.

**Fields**:
- `sequence_number: u32` - Existing message sequence (`00001`, `00002`, ...).
- `subject: String` - Existing subject line.
- `body_html: Option<String>` - Raw HTML body from `PidTagHtmlBody` when present.
- `body_plain: Option<String>` - Plain-text body fallback when present.
- `attachments: Vec<ExportAttachmentRecord>` - Exportable attachments with inline-resolution metadata.
- `matched_keywords: Vec<String>` - Existing keyword hits, now derived from visible HTML text when `body_html` is present.

**Validation rules**:
- `sequence_number` is unique within one export run.
- If `body_html` is present, keyword matching uses `SearchableHtmlBody.visible_text` instead of the raw HTML string.
- Attachments remain scoped to a single message and must not be matched across messages.

### SearchableHtmlBody

Derived representation of an HTML body for filtering purposes.

**Fields**:
- `raw_html: String` - Original HTML body used for export.
- `visible_text: String` - Parsed text content derived from visible text nodes only.
- `source_kind: enum { HtmlBody, HtmlFragment }` - Whether the original content was already a full HTML document or wrapped as a fragment.

**Validation rules**:
- `visible_text` excludes tag markup, comment text, script text, and style text.
- `visible_text` preserves normal textual content order sufficiently for substring keyword matching.

### ExportAttachmentRecord

Represents one attachment extracted from the PST message and eligible for export.

**Fields**:
- `original_filename: String` - Raw long or short attachment filename from PST properties.
- `content_type: Option<String>` - MIME type when present.
- `content_id: Option<String>` - `PidTagAttachContentId` value when present.
- `content_location: Option<String>` - `PidTagAttachContentLocation` value when present.
- `data: Vec<u8>` - Attachment payload bytes.

**Validation rules**:
- `data` must be present for a file to be exported or referenced locally.
- `content_id` and `content_location` are optional and may be absent independently.
- Metadata is scoped to the source message only.

### AttachmentExportPlan

Deterministic output plan for one message's attachments, reused by both attachment writing and HTML rewriting.

**Fields**:
- `entries: Vec<AttachmentExportPlanEntry>`
- `message_dir_relative_root: String` - The directory containing `message.html` and the attachment files.

**Validation rules**:
- Every attachment written to disk has exactly one plan entry.
- Sanitization and collision rules are applied once when the plan is built.
- Relative paths used in HTML rewriting must come from the same plan entries used for file output.

### AttachmentExportPlanEntry

Represents a single attachment's final export target.

**Fields**:
- `original_filename: String`
- `resolved_filename: String` - Sanitized and collision-resolved filename.
- `relative_path: String` - Relative path from `message.html` to the file, typically `./<resolved_filename>` or equivalent same-directory path.
- `content_id_keys: Vec<String>` - Normalized lookup keys derived from content ID.
- `content_location_keys: Vec<String>` - Normalized lookup keys derived from content-location.

**Validation rules**:
- `resolved_filename` is unique within the message directory.
- Lookup keys are normalized consistently with HTML reference normalization.

### InlineReference

Represents a URL-valued reference encountered in exported HTML.

**Fields**:
- `attribute_name: enum { src, href }`
- `original_value: String`
- `reference_kind: enum { Cid, ContentLocation, Other }`
- `normalized_lookup_key: Option<String>`
- `rewritten_value: Option<String>`

**Validation rules**:
- `reference_kind == Other` implies `rewritten_value == None`.
- `rewritten_value` is set only when exactly one same-message attachment plan entry matches the normalized lookup key.
- Multiple candidate matches are treated as unresolved.
- Unmatched references retain `original_value` in exported HTML.

## Relationships

- One `ExportMessageRecord` has zero or one `SearchableHtmlBody`.
- One `ExportMessageRecord` has zero or more `ExportAttachmentRecord` values.
- One `ExportMessageRecord` can produce one `AttachmentExportPlan`.
- One `AttachmentExportPlan` has one or more `AttachmentExportPlanEntry` values.
- One `InlineReference` may resolve to at most one `AttachmentExportPlanEntry` from the same message.

## State Transitions

1. `Extracted`: Message body and attachment PST properties are read into `ExportMessageRecord` and `ExportAttachmentRecord`.
2. `Normalized`: HTML visible text and attachment lookup keys are derived.
3. `Planned`: Attachment filenames are sanitized and collision-resolved into an `AttachmentExportPlan`.
4. `Rewritten`: HTML inline references are rewritten using the attachment export plan when a same-message match exists.
5. `Written`: `message.html` and attachment files are written using the same resolved paths.

## Invariants

- Non-HTML bodies continue to use the existing keyword matching path.
- Search hits found only in tags, comments, scripts, or styles do not count as matches.
- Attachment reference rewriting never crosses message boundaries.
- Unmatched or external references remain unchanged.
- Attachment filenames in `message.html` always match the actual exported filenames on disk.