//! HTML generation from email body formats
//!
//! Converts email body content to HTML format using a priority-based approach:
//! 1. HTML body (direct use)
//! 2. RTF body (decompress and convert to HTML)
//! 3. Plain text body (wrap with basic HTML formatting)
//!
//! Also provides parse-based visible-text extraction (for keyword filtering)
//! and inline-reference rewriting (for exported `message.html` files) via
//! `lol_html`.

use crate::error::Result;
use crate::export::exporter::AttachmentExportPlanEntry;
use encoding_rs::Encoding;
use lol_html::{element, text, HtmlRewriter, Settings};
use std::collections::HashMap;

/// Convert email body content to HTML
///
/// Attempts to produce HTML from the best available body format.
/// Priority: HTML > RTF > Plain text
///
/// # Arguments
/// * `body_html` - Preferred: HTML body content (if present)
/// * `body_rtf` - Fallback: Compressed RTF body (if present)
/// * `body_plain` - Last resort: Plain text body (if present)
/// * `charset` - Character encoding (default: UTF-8)
///
/// # Returns
/// HTML string suitable for writing to message.html
///
/// # Errors
///
/// Returns an error if RTF decompression fails or if the body content cannot be converted.
pub fn convert_to_html(
    body_html: Option<&str>,
    body_rtf: Option<&[u8]>,
    body_plain: Option<&str>,
    charset: Option<&str>,
) -> Result<String> {
    // Priority 1: Use HTML body directly
    if let Some(html) = body_html {
        return wrap_html_body(html, charset);
    }

    // Priority 2: Decompress and convert RTF
    if let Some(rtf_data) = body_rtf {
        return convert_rtf_to_html(rtf_data, charset);
    }

    // Priority 3: Wrap plain text
    if let Some(plain) = body_plain {
        return wrap_plain_text_as_html(plain, charset);
    }

    // Should not reach here - validation should ensure at least one body format
    Ok(r"<html><body><p>No message body available</p></body></html>".to_string())
}

/// Wrap HTML body with proper structure and encoding
#[allow(clippy::unnecessary_wraps)]
fn wrap_html_body(html: &str, _charset: Option<&str>) -> Result<String> {
    // Check if already wrapped in html tags
    let html_trimmed = html.trim();
    if html_trimmed.starts_with("<html") || html_trimmed.starts_with("<!DOCTYPE") {
        // Already properly structured HTML — normalize charset to UTF-8
        // so browsers don't misinterpret the (already UTF-8) Rust string.
        Ok(normalize_charset_to_utf8(html))
    } else {
        // Wrap in basic HTML structure
        Ok(format!(
            r#"<html><head><meta charset="utf-8"></head><body>{html}</body></html>"#
        ))
    }
}

/// Rewrite charset declarations in existing HTML to UTF-8.
///
/// Handles both forms:
/// - `<meta charset="...">`
/// - `<meta http-equiv="Content-Type" content="text/html; charset=...">`
///
/// Since Rust strings are always valid UTF-8, the in-memory content is
/// already UTF-8 regardless of what the original email declared. This
/// prevents browsers from misinterpreting the bytes.
fn normalize_charset_to_utf8(html: &str) -> String {
    let mut output = Vec::with_capacity(html.len());

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("meta", |el| {
                // <meta charset="...">
                if el.get_attribute("charset").is_some() {
                    el.set_attribute("charset", "utf-8").ok();
                }
                // <meta http-equiv="Content-Type" content="text/html; charset=...">
                if let Some(http_equiv) = el.get_attribute("http-equiv") {
                    if http_equiv.eq_ignore_ascii_case("Content-Type") {
                        el.set_attribute("content", "text/html; charset=utf-8")
                            .ok();
                    }
                }
                Ok(())
            })],
            ..Settings::new()
        },
        |chunk: &[u8]| {
            output.extend_from_slice(chunk);
        },
    );

    let _ = rewriter.write(html.as_bytes());
    let _ = rewriter.end();

    String::from_utf8(output).unwrap_or_else(|_| html.to_string())
}

/// Convert RTF body to HTML
fn convert_rtf_to_html(rtf_data: &[u8], _charset: Option<&str>) -> Result<String> {
    // Decompress RTF using compressed-rtf crate
    use compressed_rtf::decompress_rtf;

    let decompressed_str = decompress_rtf(rtf_data).map_err(|e| {
        crate::error::Error::Export(crate::error::ExportError::HtmlConversionFailed(format!(
            "RTF decompression failed: {e}"
        )))
    })?;

    // Basic RTF-to-HTML converter: handle common RTF tags
    let html = convert_rtf_tags_to_html(&decompressed_str);

    Ok(format!(
        r#"<html><head><meta charset="utf-8"></head><body><div style="white-space: pre-wrap;">{html}</div></body></html>"#
    ))
}

/// Convert common RTF tags to HTML equivalents
fn convert_rtf_tags_to_html(rtf: &str) -> String {
    let mut result = String::new();
    let mut chars = rtf.chars().peekable();
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_underline = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                // RTF control sequence
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'b' => {
                            chars.next(); // consume 'b'
                            if in_bold {
                                result.push_str("</b>");
                                in_bold = false;
                            } else {
                                result.push_str("<b>");
                                in_bold = true;
                            }
                        }
                        'i' => {
                            chars.next(); // consume 'i'
                            if in_italic {
                                result.push_str("</i>");
                                in_italic = false;
                            } else {
                                result.push_str("<i>");
                                in_italic = true;
                            }
                        }
                        'u' => {
                            chars.next(); // consume 'u'
                            if in_underline {
                                result.push_str("</u>");
                                in_underline = false;
                            } else {
                                result.push_str("<u>");
                                in_underline = true;
                            }
                        }
                        'p' => {
                            // \par = paragraph break
                            chars.next(); // consume 'p'
                            chars.next(); // consume 'a'
                            chars.next(); // consume 'r'
                            result.push_str("</p><p>");
                        }
                        'n' if chars.clone().nth(1) == Some('l') => {
                            // \nl = newline
                            chars.next(); // consume 'n'
                            chars.next(); // consume 'l'
                            result.push_str("<br>");
                        }
                        _ => {
                            // Skip other control sequences
                            while let Some(&c) = chars.peek() {
                                chars.next();
                                if !c.is_alphanumeric() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            '{' | '}' | '\n' | '\r' => {
                // Skip RTF group delimiters and formatting newlines
            }
            _ => {
                // Regular text
                if ch != '\0' {
                    result.push(ch);
                }
            }
        }
    }

    // Close open tags
    if in_bold {
        result.push_str("</b>");
    }
    if in_italic {
        result.push_str("</i>");
    }
    if in_underline {
        result.push_str("</u>");
    }

    html_escape::encode_text(&result).to_string()
}

/// Wrap plain text body with HTML formatting
#[allow(clippy::unnecessary_wraps)]
fn wrap_plain_text_as_html(plain: &str, _charset: Option<&str>) -> Result<String> {
    // Escape HTML special characters
    let escaped = html_escape::encode_text(plain);

    // Convert line breaks and preserve formatting
    let formatted = escaped
        .replace("\r\n", "\n")
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                "</p><p>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<String>();

    Ok(format!(
        r#"<html><head><meta charset="utf-8"></head><body><p>{formatted}</p></body></html>"#
    ))
}

/// Encode text to UTF-8 for HTML output (for character set conversion)
#[allow(dead_code)]
fn encode_to_utf8(text: &str, charset: Option<&str>) -> String {
    match charset {
        Some(cs) if cs.to_lowercase() != "utf-8" => {
            // Try to find encoding by name
            if let Some(encoding) = Encoding::for_label(cs.as_bytes()) {
                let (cow, _, _) = encoding.decode(text.as_bytes());
                cow.into_owned()
            } else {
                // Fall back to UTF-8
                text.to_string()
            }
        }
        _ => text.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Visible-text extraction (for keyword filtering on HTML bodies)
// ---------------------------------------------------------------------------

/// Extract user-visible text from an HTML string, excluding tag markup,
/// comments, `<script>`, and `<style>` content.
///
/// The returned string preserves text-node ordering and inserts spaces
/// between blocks so substring keyword matching works naturally.
pub fn extract_visible_text(html: &str) -> String {
    // Two-pass approach: first strip <script> and <style> elements completely,
    // then collect text nodes from the remaining HTML.
    let stripped = strip_hidden_elements(html);
    collect_text_nodes(&stripped)
}

/// Remove `<script>` and `<style>` elements (including content) from HTML.
fn strip_hidden_elements(html: &str) -> String {
    let mut output = Vec::with_capacity(html.len());

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("script, style", |el| {
                el.remove();
                Ok(())
            })],
            ..Settings::new()
        },
        |chunk: &[u8]| {
            output.extend_from_slice(chunk);
        },
    );

    let _ = rewriter.write(html.as_bytes());
    let _ = rewriter.end();

    String::from_utf8(output).unwrap_or_else(|_| html.to_string())
}

/// Collect all text nodes from HTML, joining with spaces.
fn collect_text_nodes(html: &str) -> String {
    let mut visible = String::with_capacity(html.len() / 2);

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![text!("*", |t| {
                let chunk = t.as_str();
                if !chunk.trim().is_empty() {
                    if !visible.is_empty() && !visible.ends_with(' ') {
                        visible.push(' ');
                    }
                    visible.push_str(chunk);
                }
                Ok(())
            })],
            ..Settings::new()
        },
        |_output: &[u8]| {},
    );

    let _ = rewriter.write(html.as_bytes());
    let _ = rewriter.end();

    visible
}

// ---------------------------------------------------------------------------
// Inline-reference rewriting (for exported message.html)
// ---------------------------------------------------------------------------

/// Classify an attribute value as an inline reference kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineReferenceKind {
    /// `cid:` reference.
    Cid,
    /// Content-location based reference.
    ContentLocation,
    /// External URL, anchor, or other non-inline reference.
    Other,
}

/// Classify a URL-valued attribute as a `cid:` reference, a potential
/// content-location reference, or something else (external / anchor).
pub fn classify_reference(value: &str) -> InlineReferenceKind {
    let trimmed = value.trim();
    if trimmed.to_ascii_lowercase().starts_with("cid:") {
        return InlineReferenceKind::Cid;
    }
    // External URLs and anchors are "Other"
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with('#')
        || trimmed.starts_with("data:")
    {
        return InlineReferenceKind::Other;
    }
    // Any remaining non-empty value could be a content-location reference.
    if !trimmed.is_empty() {
        return InlineReferenceKind::ContentLocation;
    }
    InlineReferenceKind::Other
}

/// Normalize a `cid:` reference value into a lookup key that matches our
/// attachment plan's `content_id_keys`.
pub fn normalize_cid_key(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("cid:")
        .trim_start_matches("CID:")
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_lowercase()
}

/// Normalize a content-location reference value into a lookup key.
pub fn normalize_content_location_key(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Build lookup maps from an attachment export plan for fast reference resolution.
fn build_lookup_maps(
    plan_entries: &[AttachmentExportPlanEntry],
) -> (HashMap<String, Vec<usize>>, HashMap<String, Vec<usize>>) {
    let mut cid_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut loc_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, entry) in plan_entries.iter().enumerate() {
        for key in &entry.content_id_keys {
            cid_map.entry(key.clone()).or_default().push(i);
        }
        for key in &entry.content_location_keys {
            loc_map.entry(key.clone()).or_default().push(i);
        }
    }

    (cid_map, loc_map)
}

/// Resolve an inline reference to an attachment plan entry index.
///
/// Returns `Some(plan_index)` only when exactly one attachment matches.
/// Multiple candidates are treated as unresolved (returns `None`).
fn resolve_reference(
    value: &str,
    kind: InlineReferenceKind,
    cid_map: &HashMap<String, Vec<usize>>,
    loc_map: &HashMap<String, Vec<usize>>,
) -> Option<usize> {
    match kind {
        InlineReferenceKind::Cid => {
            let key = normalize_cid_key(value);
            cid_map.get(&key).and_then(|indices| {
                if indices.len() == 1 {
                    Some(indices[0])
                } else {
                    None // ambiguous
                }
            })
        }
        InlineReferenceKind::ContentLocation => {
            let key = normalize_content_location_key(value);
            loc_map.get(&key).and_then(|indices| {
                if indices.len() == 1 {
                    Some(indices[0])
                } else {
                    None // ambiguous
                }
            })
        }
        InlineReferenceKind::Other => None,
    }
}

/// Rewrite inline `src` and `href` references in HTML to local attachment paths.
///
/// Only references that unambiguously match a single attachment in the plan
/// are rewritten. External URLs, anchor links, and unresolved references
/// remain untouched.
pub fn rewrite_inline_references(
    html: &str,
    plan_entries: &[AttachmentExportPlanEntry],
) -> String {
    if plan_entries.is_empty() {
        return html.to_string();
    }

    let (cid_map, loc_map) = build_lookup_maps(plan_entries);

    let mut output = Vec::with_capacity(html.len());

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("*", |el| {
                for attr_name in &["src", "href"] {
                    if let Some(value) = el.get_attribute(attr_name) {
                        let kind = classify_reference(&value);
                        if kind == InlineReferenceKind::Other {
                            continue;
                        }
                        if let Some(plan_idx) = resolve_reference(&value, kind, &cid_map, &loc_map)
                        {
                            el.set_attribute(attr_name, &plan_entries[plan_idx].relative_path)
                                .ok();
                        }
                    }
                }
                Ok(())
            })],
            ..Settings::new()
        },
        |chunk: &[u8]| {
            output.extend_from_slice(chunk);
        },
    );

    let _ = rewriter.write(html.as_bytes());
    let _ = rewriter.end();

    String::from_utf8(output).unwrap_or_else(|_| html.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_body_passthrough() {
        let html = "<html><body>Test content</body></html>";
        let result = convert_to_html(Some(html), None, None, None).unwrap();
        assert!(result.contains("Test content"));
    }

    #[test]
    fn test_plain_text_wrapping() {
        let plain = "Line 1\nLine 2\nLine 3";
        let result = convert_to_html(None, None, Some(plain), None).unwrap();
        assert!(result.contains("<html>"));
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 2"));
    }

    #[test]
    fn test_plain_text_html_escaping() {
        let plain = "Test <script>alert('xss')</script>";
        let result = convert_to_html(None, None, Some(plain), None).unwrap();
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_priority_html_over_rtf() {
        let html = "<html><body>HTML content</body></html>";
        let result = convert_to_html(Some(html), None, None, None).unwrap();
        assert!(result.contains("HTML content"));
    }
}
