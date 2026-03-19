# CLI Interface Contract: HTML Search and Inline Export

**Feature**: [../spec.md](../spec.md) | [../plan.md](../plan.md) | [../data-model.md](../data-model.md)  
**Phase**: 1 - Design  
**Date**: 2026-03-09

## Overview

This contract defines the external `pst-cli export` behavior for HTML-aware keyword filtering and inline attachment reference rewriting.

## Command Surface Change

### Command
`pst-cli export`

### New Flags
None.

### Existing Flags Affected
- `--keywords`
- `--attachments`
- `--metadata`

## Keyword Filtering Contract

When `--keywords` is provided:
- Subject matching behavior remains unchanged.
- Plain-text body matching behavior remains unchanged.
- If an HTML body is present, keyword matching MUST use user-visible HTML text rather than the raw HTML source.
- Text appearing only inside tags, comments, `<script>`, or `<style>` blocks MUST NOT count as a match.

When `--keywords` is not provided:
- No keyword matching is performed, as today.

## HTML Export Contract

When `message.html` is generated:
- The exported file remains the canonical HTML representation of the message.
- If the message contains attachments exported for the same message directory and an HTML reference can be matched to one of them by `cid:` or content-location, the HTML reference is rewritten to the corresponding relative local path.
- Rewriting applies to standard URL-valued HTML attributes used for inline resources, starting with `src` and `href`.
- Attachment filename sanitization and collision handling used in rewritten HTML MUST match the filenames actually written to disk.

## Attachment and Link Preservation Contract

- Rewriting is scoped to attachments exported for the same message only.
- External URLs, anchor links, and unrelated HTML references remain unchanged.
- If attachment export is disabled or a reference cannot be resolved, the original HTML reference remains unchanged.
- Export continues successfully even when some inline references are unresolved.

## Output Layout Contract

### Baseline (existing)
- Message HTML: `<output>/<seq>/message.html`
- Attachments: `<output>/<seq>/<sanitized-filename>`

### With conversation grouping already enabled elsewhere
- Grouped message HTML: `<output>/conv_#####/<seq>/message.html`
- Grouped attachments: `<output>/conv_#####/<seq>/<sanitized-filename>`

### Rewrite behavior
- Rewritten inline references in `message.html` target a relative path to the attachment file in the same message directory.
- The relative path resolves to the exact exported filename after sanitization and collision handling.

## Determinism Contract

For a fixed input set and unchanged traversal order:
- Keyword matching results are deterministic.
- Attachment filename planning is deterministic.
- Inline reference rewriting is deterministic and reflects the exact files exported for that message.
- Each exported message still produces exactly one `message.html` file.

## Validation Scenarios

1. An HTML message contains `invoice` only inside `<script>`.
- Expect no keyword hit for `invoice`.

2. An HTML message contains `invoice` in visible paragraph text.
- Expect a keyword hit for `invoice`.

3. An HTML message contains `<img src="cid:logo123">` and an exported attachment with matching content ID.
- Expect `message.html` to reference the local exported attachment filename.

4. An HTML message contains a content-location reference matching an exported attachment.
- Expect `message.html` to reference the local exported attachment filename.

5. An HTML message contains `https://example.com/logo.png` and no matching local attachment.
- Expect the external URL to remain unchanged.

6. An HTML message contains `cid:missing-image` but no exported attachment matches it.
- Expect the original `cid:` reference to remain unchanged and export to succeed.