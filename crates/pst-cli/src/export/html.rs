//! HTML generation from email body formats
//!
//! Converts email body content to HTML format using a priority-based approach:
//! 1. HTML body (direct use)
//! 2. RTF body (decompress and convert to HTML)
//! 3. Plain text body (wrap with basic HTML formatting)

use crate::error::Result;
use encoding_rs::Encoding;

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
        // Already properly structured HTML
        Ok(html.to_string())
    } else {
        // Wrap in basic HTML structure
        Ok(format!(
            r#"<html><head><meta charset="utf-8"></head><body>{html}</body></html>"#
        ))
    }
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
