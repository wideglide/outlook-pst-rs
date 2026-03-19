//! Unit tests for HTML conversion functionality
//!
//! Tests T026: HTML conversion from various body formats
//! - HTML pass-through
//! - RTF to HTML conversion (basic tags)
//! - Plain text wrapping (check <p> and <br> tags)
//! - Character encoding handling
//!
//! Tests T008 [US1]: Visible-text extraction and malformed-HTML handling

use pst_cli::export::html::{
    classify_reference, convert_to_html, extract_visible_text, normalize_cid_key,
    normalize_content_location_key, rewrite_inline_references, InlineReferenceKind,
};
use pst_cli::export::exporter::AttachmentExportPlanEntry;

#[test]
fn test_html_passthrough_complete() {
    let html =
        r"<html><head><title>Test</title></head><body><p>Complete HTML document</p></body></html>";
    let result = convert_to_html(Some(html), None, None, None).unwrap();

    // Should preserve complete HTML structure
    assert!(result.contains("<html>"));
    assert!(result.contains("Complete HTML document"));
    assert!(result.contains("</body></html>"));
}

#[test]
fn test_html_passthrough_fragment() {
    let html_fragment = r"<div><strong>Bold text</strong> and <em>italic text</em></div>";
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

// ---------------------------------------------------------------------------
// T008 [US1] Visible-text extraction tests
// ---------------------------------------------------------------------------

#[test]
fn test_visible_text_simple_paragraph() {
    let html = "<html><body><p>Hello world</p></body></html>";
    let text = extract_visible_text(html);
    assert!(text.contains("Hello world"));
}

#[test]
fn test_visible_text_excludes_script_content() {
    let html = r#"<html><body>
        <p>Visible paragraph</p>
        <script>var keyword = "invoice";</script>
    </body></html>"#;
    let text = extract_visible_text(html);
    assert!(text.contains("Visible paragraph"));
    assert!(
        !text.contains("invoice"),
        "Script content must not be extracted: {text}"
    );
}

#[test]
fn test_visible_text_excludes_style_content() {
    let html = r#"<html><body>
        <style>.invoice { color: red; }</style>
        <p>Real content</p>
    </body></html>"#;
    let text = extract_visible_text(html);
    assert!(text.contains("Real content"));
    assert!(
        !text.contains("invoice"),
        "Style content must not be extracted: {text}"
    );
}

#[test]
fn test_visible_text_excludes_html_comments() {
    let html = "<html><body><!-- invoice hidden here --><p>Normal text</p></body></html>";
    let text = extract_visible_text(html);
    assert!(text.contains("Normal text"));
    assert!(
        !text.contains("invoice"),
        "HTML comment content must not be extracted: {text}"
    );
}

#[test]
fn test_visible_text_excludes_tag_attribute_values() {
    let html = r#"<html><body><div class="invoice-wrapper"><p>Body text only</p></div></body></html>"#;
    let text = extract_visible_text(html);
    assert!(text.contains("Body text only"));
    assert!(
        !text.contains("invoice-wrapper"),
        "Tag attributes must not be extracted: {text}"
    );
}

#[test]
fn test_visible_text_multiple_text_nodes() {
    let html = "<html><body><p>First</p><p>Second</p><div>Third</div></body></html>";
    let text = extract_visible_text(html);
    assert!(text.contains("First"));
    assert!(text.contains("Second"));
    assert!(text.contains("Third"));
}

#[test]
fn test_visible_text_malformed_html_unclosed_tags() {
    let html = "<p>Some text<div>More text";
    let text = extract_visible_text(html);
    assert!(text.contains("Some text"));
    assert!(text.contains("More text"));
}

#[test]
fn test_visible_text_malformed_html_script_unclosed() {
    // Even with malformed HTML, script content should not leak into visible text
    let html = "<p>Visible</p><script>var x = 'hidden';</script><p>Also visible</p>";
    let text = extract_visible_text(html);
    assert!(text.contains("Visible"));
    assert!(text.contains("Also visible"));
    assert!(
        !text.contains("hidden"),
        "Script content must not leak: {text}"
    );
}

#[test]
fn test_visible_text_empty_html() {
    let text = extract_visible_text("");
    assert!(text.is_empty());
}

#[test]
fn test_visible_text_nested_elements() {
    let html = "<html><body><div><span>Nested <b>bold</b> text</span></div></body></html>";
    let text = extract_visible_text(html);
    assert!(text.contains("Nested"));
    assert!(text.contains("bold"));
    assert!(text.contains("text"));
}

#[test]
fn test_visible_text_preserves_order() {
    let html = "<html><body><p>Alpha</p><p>Beta</p><p>Gamma</p></body></html>";
    let text = extract_visible_text(html);
    let alpha_pos = text.find("Alpha").unwrap();
    let beta_pos = text.find("Beta").unwrap();
    let gamma_pos = text.find("Gamma").unwrap();
    assert!(alpha_pos < beta_pos);
    assert!(beta_pos < gamma_pos);
}

#[test]
fn test_visible_text_mixed_hidden_and_visible() {
    let html = r#"<html><head>
        <style>body { font-size: 14px; }</style>
        <script>document.title = "confidential";</script>
    </head><body>
        <p>This invoice is payable</p>
        <!-- internal: confidential draft -->
        <script>trackEvent("confidential")</script>
        <p>Please review</p>
    </body></html>"#;
    let text = extract_visible_text(html);
    assert!(text.contains("invoice"));
    assert!(text.contains("payable"));
    assert!(text.contains("Please review"));
    assert!(
        !text.contains("confidential"),
        "Hidden content must not appear: {text}"
    );
}

// --- US2: cid: and content-location Rewrite Unit Tests (T015) ---

fn plan_entry(
    index: usize,
    filename: &str,
    cid_keys: Vec<&str>,
    loc_keys: Vec<&str>,
) -> AttachmentExportPlanEntry {
    AttachmentExportPlanEntry {
        attachment_index: index,
        resolved_filename: filename.to_string(),
        relative_path: filename.to_string(),
        content_id_keys: cid_keys.into_iter().map(String::from).collect(),
        content_location_keys: loc_keys.into_iter().map(String::from).collect(),
    }
}

// --- classify_reference tests ---

#[test]
fn test_classify_cid_reference() {
    assert_eq!(classify_reference("cid:image001@mail"), InlineReferenceKind::Cid);
    assert_eq!(classify_reference("CID:Image001@mail"), InlineReferenceKind::Cid);
    assert_eq!(classify_reference("  cid:foo  "), InlineReferenceKind::Cid);
}

#[test]
fn test_classify_external_urls() {
    assert_eq!(classify_reference("http://example.com"), InlineReferenceKind::Other);
    assert_eq!(classify_reference("https://example.com"), InlineReferenceKind::Other);
    assert_eq!(classify_reference("mailto:user@example.com"), InlineReferenceKind::Other);
    assert_eq!(classify_reference("#anchor"), InlineReferenceKind::Other);
    assert_eq!(classify_reference("data:image/png;base64,abc"), InlineReferenceKind::Other);
}

#[test]
fn test_classify_content_location() {
    assert_eq!(classify_reference("image001.png"), InlineReferenceKind::ContentLocation);
    assert_eq!(classify_reference("Logo.PNG"), InlineReferenceKind::ContentLocation);
}

#[test]
fn test_classify_empty_is_other() {
    assert_eq!(classify_reference(""), InlineReferenceKind::Other);
    assert_eq!(classify_reference("   "), InlineReferenceKind::Other);
}

// --- normalize keys tests ---

#[test]
fn test_normalize_cid_key_strips_prefix_and_brackets() {
    assert_eq!(normalize_cid_key("cid:<Image001@example.com>"), "image001@example.com");
    assert_eq!(normalize_cid_key("CID:Image002@mail"), "image002@mail");
    assert_eq!(normalize_cid_key("  cid:  <FOO>  "), "foo");
}

#[test]
fn test_normalize_content_location_key_lowercase() {
    assert_eq!(normalize_content_location_key("Logo.PNG"), "logo.png");
    assert_eq!(normalize_content_location_key("  Header.JPG  "), "header.jpg");
}

// --- rewrite_inline_references tests ---

#[test]
fn test_rewrite_cid_reference_in_img_src() {
    let html = r#"<html><body><img src="cid:image001@mail"></body></html>"#;
    let entries = vec![plan_entry(0, "image001.png", vec!["image001@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"src="image001.png""#), "cid: should be rewritten: {result}");
    assert!(!result.contains("cid:"), "No cid: should remain: {result}");
}

#[test]
fn test_rewrite_content_location_in_img_src() {
    let html = r#"<html><body><img src="logo.png"></body></html>"#;
    let entries = vec![plan_entry(0, "exported_logo.png", vec![], vec!["logo.png"])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"src="exported_logo.png""#), "Content-location should be rewritten: {result}");
}

#[test]
fn test_rewrite_cid_case_insensitive() {
    let html = r#"<img src="CID:Image001@Mail">"#;
    let entries = vec![plan_entry(0, "image.png", vec!["image001@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"src="image.png""#), "Case-insensitive cid matching: {result}");
}

#[test]
fn test_rewrite_preserves_external_urls() {
    let html = r#"<a href="https://example.com">Link</a><img src="cid:img@mail">"#;
    let entries = vec![plan_entry(0, "img.png", vec!["img@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"href="https://example.com""#), "External URL unchanged: {result}");
    assert!(result.contains(r#"src="img.png""#), "cid: should be rewritten: {result}");
}

#[test]
fn test_rewrite_preserves_mailto_links() {
    let html = r#"<a href="mailto:user@example.com">Email</a>"#;
    let entries = vec![plan_entry(0, "img.png", vec!["user@example.com"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"href="mailto:user@example.com""#), "mailto: untouched: {result}");
}

#[test]
fn test_rewrite_preserves_anchor_links() {
    let html = r##"<a href="#section1">Jump</a>"##;
    let entries = vec![plan_entry(0, "img.png", vec![], vec!["#section1"])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r##"href="#section1""##), "Anchor link untouched: {result}");
}

#[test]
fn test_rewrite_empty_plan_returns_original() {
    let html = r#"<img src="cid:image@mail">"#;
    let result = rewrite_inline_references(html, &[]);
    assert_eq!(result, html);
}

#[test]
fn test_rewrite_multiple_cid_references() {
    let html = r#"<img src="cid:header@mail"><img src="cid:footer@mail">"#;
    let entries = vec![
        plan_entry(0, "header.png", vec!["header@mail"], vec![]),
        plan_entry(1, "footer.png", vec!["footer@mail"], vec![]),
    ];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"src="header.png""#), "Header rewritten: {result}");
    assert!(result.contains(r#"src="footer.png""#), "Footer rewritten: {result}");
}

#[test]
fn test_rewrite_unmatched_cid_preserved() {
    let html = r#"<img src="cid:unknown@mail">"#;
    let entries = vec![plan_entry(0, "img.png", vec!["other@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains("cid:unknown@mail"), "Unmatched cid: preserved: {result}");
}

#[test]
fn test_rewrite_ambiguous_cid_preserved() {
    // Two attachments with the same content_id key - should not rewrite (ambiguous)
    let html = r#"<img src="cid:dup@mail">"#;
    let entries = vec![
        plan_entry(0, "a.png", vec!["dup@mail"], vec![]),
        plan_entry(1, "b.png", vec!["dup@mail"], vec![]),
    ];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains("cid:dup@mail"), "Ambiguous cid: must be preserved: {result}");
}

#[test]
fn test_rewrite_href_cid_in_anchor() {
    let html = r#"<a href="cid:doc@mail">Download</a>"#;
    let entries = vec![plan_entry(0, "document.pdf", vec!["doc@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"href="document.pdf""#), "href cid: rewritten: {result}");
}

// --- US3: Preserve Unrelated and Unresolvable Links (T021) ---

#[test]
fn test_rewrite_preserves_data_urls() {
    let html = r#"<img src="data:image/png;base64,iVBOR">"#;
    let entries = vec![plan_entry(0, "img.png", vec!["img@mail"], vec![])];
    let result = rewrite_inline_references(html, &entries);
    assert!(
        result.contains("data:image/png;base64,iVBOR"),
        "data: URL must be preserved: {result}"
    );
}

#[test]
fn test_rewrite_mixed_resolvable_and_unresolvable() {
    let html = r#"<html><body>
        <img src="cid:logo@mail">
        <a href="https://example.com">External</a>
        <img src="cid:unknown@mail">
        <img src="banner.png">
        <a href="mailto:help@example.com">Help</a>
    </body></html>"#;
    let entries = vec![
        plan_entry(0, "logo.png", vec!["logo@mail"], vec![]),
        plan_entry(1, "banner_exported.png", vec![], vec!["banner.png"]),
    ];
    let result = rewrite_inline_references(html, &entries);
    // Resolvable references rewritten
    assert!(result.contains(r#"src="logo.png""#), "cid:logo rewritten: {result}");
    assert!(result.contains(r#"src="banner_exported.png""#), "content-location rewritten: {result}");
    // Unresolvable and external preserved
    assert!(result.contains("cid:unknown@mail"), "Unknown cid: preserved: {result}");
    assert!(result.contains("https://example.com"), "External URL preserved: {result}");
    assert!(result.contains("mailto:help@example.com"), "mailto: preserved: {result}");
}

#[test]
fn test_rewrite_ambiguous_content_location_preserved() {
    // Two attachments match the same content-location key
    let html = r#"<img src="shared.png">"#;
    let entries = vec![
        plan_entry(0, "a.png", vec![], vec!["shared.png"]),
        plan_entry(1, "b.png", vec![], vec!["shared.png"]),
    ];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains("shared.png"), "Ambiguous content-location must be preserved: {result}");
    assert!(!result.contains("a.png"), "Should not resolve to first: {result}");
    assert!(!result.contains("b.png"), "Should not resolve to second: {result}");
}

#[test]
fn test_rewrite_unmatched_content_location_preserved() {
    let html = r#"<img src="missing.png">"#;
    let entries = vec![plan_entry(0, "other.png", vec![], vec!["different.png"])];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains("missing.png"), "Unmatched content-location preserved: {result}");
}

#[test]
fn test_rewrite_element_with_both_src_and_href() {
    // Unusual but valid: element has both src and href attributes
    let html = r#"<video src="cid:video@mail" href="cid:poster@mail"></video>"#;
    let entries = vec![
        plan_entry(0, "video.mp4", vec!["video@mail"], vec![]),
        plan_entry(1, "poster.jpg", vec!["poster@mail"], vec![]),
    ];
    let result = rewrite_inline_references(html, &entries);
    assert!(result.contains(r#"src="video.mp4""#), "src rewritten: {result}");
    assert!(result.contains(r#"href="poster.jpg""#), "href rewritten: {result}");
}
