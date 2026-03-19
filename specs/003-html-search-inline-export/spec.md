# Feature Specification: HTML Search and Inline Export

**Feature Branch**: `[003-html-search-inline-export]`  
**Created**: 2026-03-09  
**Status**: Draft  
**Input**: User description: "HTML parsing during keyword search so that keywords are not identified within html tags, script blocks, style blocks, and comments. Native inline attachment resolution to pst-cli export so exported message.html rewrites inline image and attachment references such as cid: and content-location references to the exported local files on disk. Prefer a parse-based HTML rewriter using lol_html or html5ever rather than regex replacement. Avoid excessive code or overly complex implementation."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Search Visible HTML Content Only (Priority: P1)

As an investigator running keyword searches during export, I want HTML message bodies to be searched based on visible content only so that search results reflect what a person would actually read in the message.

**Why this priority**: False positives in keyword search directly reduce trust in export results and can cause users to review irrelevant messages.

**Independent Test**: Export a dataset containing HTML messages where target keywords appear in visible body text, tag markup, script blocks, style blocks, and comments, then verify only messages with visible-body matches are selected.

**Acceptance Scenarios**:

1. **Given** an HTML message where a keyword appears in visible message text, **When** export filtering is run with that keyword, **Then** the message is included as a keyword match.
2. **Given** an HTML message where a keyword appears only inside markup, script content, style content, or HTML comments, **When** export filtering is run with that keyword, **Then** the message is not included as a keyword match.

---

### User Story 2 - Resolve Inline References in Exported HTML (Priority: P2)

As a user opening exported HTML messages from disk, I want inline images and embedded attachment links to resolve to exported local files so that the exported message renders correctly without manual fixes.

**Why this priority**: Exported HTML that does not resolve inline resources loses evidence value and is harder to review offline.

**Independent Test**: Export messages containing inline attachments referenced by `cid:` or content-location values, open the generated `message.html`, and verify the referenced resources load from exported local files.

**Acceptance Scenarios**:

1. **Given** a message with an inline image referenced by `cid:` and that attachment is exported, **When** `message.html` is generated, **Then** the inline reference points to the corresponding exported local file.
2. **Given** a message with HTML that references an attachment by content-location and that attachment is exported, **When** `message.html` is generated, **Then** the reference points to the corresponding exported local file.

---

### User Story 3 - Preserve Unrelated and Unresolvable Links (Priority: P3)

As a user reviewing exported messages, I want the export to rewrite only references that can be tied to exported attachments so that unrelated URLs and unresolved references are preserved rather than corrupted.

**Why this priority**: Overwriting unrelated links or partially matching the wrong attachment would damage message fidelity and make exports less reliable.

**Independent Test**: Export messages containing a mix of external links, unresolvable inline references, and resolvable inline references, then verify only the resolvable inline references are changed.

**Acceptance Scenarios**:

1. **Given** a `message.html` file containing both external web URLs and resolvable inline attachment references, **When** export completes, **Then** only the resolvable inline attachment references are rewritten to local files.
2. **Given** a `message.html` file containing an inline reference with no matching exported attachment, **When** export completes, **Then** the original reference remains unchanged and the export still succeeds.

### Edge Cases

- HTML may be malformed or incomplete; visible-text keyword matching must still avoid counting keywords that appear only inside markup, scripts, styles, or comments.
- The same inline attachment may be referenced multiple times in one message; each reference must resolve consistently to the same exported local file.
- Multiple attachments may have similar names while only one matches a given inline reference; the system must rewrite only the reference that can be matched unambiguously.
- A message may contain inline references when attachment export is disabled or when the referenced attachment is unavailable; export must still produce `message.html` without inventing replacement paths.
- External web URLs and anchor links may appear alongside inline attachment references; they must remain unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST evaluate keyword matches in HTML message bodies using only user-visible content.
- **FR-002**: System MUST NOT treat text found only inside HTML tags as a keyword match.
- **FR-003**: System MUST NOT treat text found only inside HTML script blocks, style blocks, or comments as a keyword match.
- **FR-004**: System MUST preserve existing keyword-search behavior for non-HTML message bodies.
- **FR-005**: When `message.html` is generated and an inline attachment reference can be matched to an exported attachment file, the system MUST rewrite that reference to the exported local file path.
- **FR-006**: System MUST support rewriting inline references expressed as `cid:` values.
- **FR-007**: System MUST support rewriting inline references expressed as content-location values.
- **FR-008**: System MUST rewrite only references that match attachments exported for the same message.
- **FR-009**: System MUST preserve external URLs, anchor links, and unrelated HTML references without modification.
- **FR-010**: When an inline reference cannot be matched to an exported attachment file, the system MUST leave the original reference unchanged.
- **FR-011**: Rewriting inline references MUST NOT require manual post-processing of the exported HTML.
- **FR-012**: Each exported message MUST continue to produce exactly one `message.html` file when HTML export is enabled.
- **FR-013**: System MUST extract and retain enough attachment metadata to resolve inline HTML references, including `PidTagAttachContentId` when present and `PidTagAttachContentLocation` when present.
- **FR-014**: System MUST determine final exported attachment filenames before writing `message.html` so HTML rewriting and file output use the same attachment plan.
- **FR-015**: System MUST apply the same filename sanitization and collision-handling rules to inline attachment references as are used for exported attachment files.
- **FR-016**: System MUST rewrite resolvable `cid:` references in HTML to relative local file paths pointing to the exported attachment files in the same message output directory.
- **FR-017**: System MUST rewrite resolvable content-location-based inline references in HTML to relative local file paths pointing to the exported attachment files in the same message output directory.
- **FR-018**: System MUST leave unresolved inline references unchanged rather than guessing or fabricating a match.


### Key Entities *(include if feature involves data)*

- **HTML Message Body**: The HTML representation of a message used for keyword evaluation and exported `message.html` output.
- **Keyword Match Candidate**: A text occurrence encountered during filtering that may or may not count as a true keyword match depending on whether it appears in user-visible content.
- **Inline Attachment Reference**: A link or resource reference in HTML that points to an embedded message attachment, including `cid:` and content-location forms.
- **Inline Attachment Metadata**: Attachment-level PST properties used to match an exported file to an HTML reference, including content ID, content location, filename, MIME type, and attachment payload availability.
- **Exported Attachment File**: A file written to disk during export that may serve as the local target for an inline attachment reference.

### Assumptions

- Keyword filtering already supports HTML message bodies and this feature refines how those bodies are interpreted for matching.
- Inline reference rewriting is relevant only when a message produces `message.html` and the referenced attachment is also exported.
- Existing export structure and attachment naming remain unchanged except for the updated HTML references.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a validation corpus of HTML messages containing known visible-text keyword hits, 100% of messages with visible-text hits are selected by keyword filtering.
- **SC-002**: In a validation corpus where keywords appear only in tags, script blocks, style blocks, or comments, 0 such messages are selected as keyword matches.
- **SC-003**: In validation exports containing resolvable inline `cid:` and content-location references, 100% of those references resolve to exported local files in the generated `message.html` output.
- **SC-004**: In validation exports containing external URLs and unresolved inline references, 100% of those references remain unchanged after export.
- **SC-005**: In validation exports containing inline attachments, 100% of exported HTML messages remain openable from disk without requiring manual link edits.
