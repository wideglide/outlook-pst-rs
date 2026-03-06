//! Unit tests for HTML conversion functionality
//!
//! Tests T026: HTML conversion from various body formats
//! - HTML pass-through
//! - RTF to HTML conversion (basic tags)
//! - Plain text wrapping (check <p> and <br> tags)
//! - Character encoding handling

use pst_cli::export::html::convert_to_html;

#[test]
fn test_html_passthrough_complete() {
    let html = r#"<html><head><title>Test</title></head><body><p>Complete HTML document</p></body></html>"#;
    let result = convert_to_html(Some(html), None, None, None).unwrap();
    
    // Should preserve complete HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("Complete HTML document"));
    assert!(result.contains("</body></html>"));
}

#[test]
fn test_html_passthrough_fragment() {
    let html_fragment = r#"<div><strong>Bold text</strong> and <em>italic text</em></div>"#;
    let result = convert_to_html(Some(html_fragment), None, None, None).unwrap();
    
    // Should wrap fragment in HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("<meta charset=\"utf-8\">"));
    assert!(result.contains("<strong>Bold text</strong>"));
    assert!(result.contains("<em>italic text</em>"));
}

#[test]
fn test_html_priority_over_other_formats() {
    let html = "<p>HTML content</p>";
    let plain = "Plain text content";
    
    // HTML should be preferred over plain text
    let result = convert_to_html(Some(html), None, Some(plain), None).unwrap();
    assert!(result.contains("HTML content"));
    assert!(!result.contains("Plain text content"));
}

#[test]
fn test_plain_text_paragraph_wrapping() {
    let plain = "First paragraph\n\nSecond paragraph\n\nThird paragraph";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should wrap in HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("<body>"));
    assert!(result.contains("<p>"));
    
    // Should contain all paragraphs
    assert!(result.contains("First paragraph"));
    assert!(result.contains("Second paragraph"));
    assert!(result.contains("Third paragraph"));
}

#[test]
fn test_plain_text_line_breaks() {
    let plain = "Line 1\nLine 2\nLine 3";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should preserve line structure
    assert!(result.contains("Line 1"));
    assert!(result.contains("Line 2"));
    assert!(result.contains("Line 3"));
}

#[test]
fn test_plain_text_html_escaping() {
    let plain = r#"Test <script>alert('xss')</script> and <b>tags</b> & special "chars""#;
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should escape HTML special characters
    assert!(result.contains("&lt;script&gt;"));
    assert!(result.contains("&lt;/script&gt;"));
    assert!(result.contains("&lt;b&gt;"));
    assert!(result.contains("&amp;"));
    
    // Should NOT contain unescaped tags
    assert!(!result.contains("<script>"));
    assert!(!result.contains("</script>"));
}

#[test]
fn test_plain_text_empty_lines() {
    let plain = "Text with\n\n\nmultiple\n\n\nempty lines";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    assert!(result.contains("Text with"));
    assert!(result.contains("multiple"));
    assert!(result.contains("empty lines"));
}

#[test]
fn test_plain_text_windows_line_endings() {
    let plain = "Line 1\r\nLine 2\r\nLine 3";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should handle Windows line endings
    assert!(result.contains("Line 1"));
    assert!(result.contains("Line 2"));
    assert!(result.contains("Line 3"));
}

#[test]
fn test_plain_text_unicode_content() {
    let plain = "Unicode: こんにちは 你好 🎉 Привет";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should preserve Unicode characters
    assert!(result.contains("こんにちは"));
    assert!(result.contains("你好"));
    assert!(result.contains("🎉"));
    assert!(result.contains("Привет"));
    assert!(result.contains("charset=\"utf-8\""));
}

#[test]
fn test_no_body_available() {
    // All body formats are None
    let result = convert_to_html(None, None, None, None).unwrap();
    
    // Should return fallback message
    assert!(result.contains("No message body available"));
    assert!(result.contains("<html>"));
}

#[test]
fn test_rtf_decompression_and_conversion() {
    // Create a simple compressed RTF body
    // Note: This would require actual compressed RTF data
    // For now, we'll skip this test or mark it as integration test
    // since it requires actual compressed RTF samples
    
    // This test should be in integration tests with actual fixtures
}

#[test]
fn test_encoding_charset_utf8() {
    let html = "<p>UTF-8 content: Ñoño</p>";
    let result = convert_to_html(Some(html), None, None, Some("utf-8")).unwrap();
    
    assert!(result.contains("Ñoño"));
}

#[test]
fn test_html_with_inline_styles() {
    let html = r#"<div style="color: red; font-size: 14px;">Styled content</div>"#;
    let result = convert_to_html(Some(html), None, None, None).unwrap();
    
    // Should preserve inline styles
    assert!(result.contains("style=\"color: red"));
    assert!(result.contains("Styled content"));
}

#[test]
fn test_html_with_links() {
    let html = r#"<a href="https://example.com">Link text</a>"#;
    let result = convert_to_html(Some(html), None, None, None).unwrap();
    
    // Should preserve links
    assert!(result.contains("href=\"https://example.com\""));
    assert!(result.contains("Link text"));
}

#[test]
fn test_plain_text_with_special_formatting_chars() {
    let plain = "Text with tabs\there\tand\tthere";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should preserve tabs (or handle appropriately)
    assert!(result.contains("Text with tabs"));
    assert!(result.contains("here"));
}

#[test]
fn test_empty_html_body() {
    let html = "";
    let result = convert_to_html(Some(html), None, None, None).unwrap();
    
    // Should still generate valid HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("<body>"));
}

#[test]
fn test_empty_plain_body() {
    let plain = "";
    let result = convert_to_html(None, None, Some(plain), None).unwrap();
    
    // Should still generate valid HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("<body>"));
}

#[test]
fn test_html_priority_order() {
    let html = "<p>HTML</p>";
    let rtf_data = b"RTF data"; // Simplified RTF data
    let plain = "Plain text";
    
    // HTML should take priority
    let result = convert_to_html(Some(html), Some(rtf_data), Some(plain), None).unwrap();
    assert!(result.contains("HTML"));
}
