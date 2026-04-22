//! Unit tests for attachment export functionality

use pst_cli::export::exporter::{build_attachment_plan, Attachment};

#[test]
fn test_attachment_creation() {
    let attachment = Attachment {
        filename: "document.pdf".to_string(),
        data: vec![0x25, 0x50, 0x44, 0x46], // PDF header: %PDF
        content_type: Some("application/pdf".to_string()),
        content_id: None,
        content_location: None,
    };

    assert_eq!(attachment.filename, "document.pdf");
    assert_eq!(attachment.data.len(), 4);
    assert_eq!(attachment.content_type, Some("application/pdf".to_string()));
}

#[test]
fn test_attachment_without_content_type() {
    let attachment = Attachment {
        filename: "unknown_file.dat".to_string(),
        data: vec![1, 2, 3, 4, 5],
        content_type: None,
        content_id: None,
        content_location: None,
    };

    assert_eq!(attachment.filename, "unknown_file.dat");
    assert_eq!(attachment.data.len(), 5);
    assert_eq!(attachment.content_type, None);
}

#[test]
fn test_attachment_with_special_chars_in_filename() {
    let attachment = Attachment {
        filename: "my document (final).docx".to_string(),
        data: vec![0x50, 0x4B, 0x03, 0x04], // ZIP header (DOCX is a ZIP)
        content_type: Some(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        ),
        content_id: None,
        content_location: None,
    };

    assert_eq!(attachment.filename, "my document (final).docx");
    assert!(attachment.filename.contains(' '));
    assert!(attachment.filename.contains('('));
    assert!(attachment.filename.contains(')'));
}

#[test]
fn test_attachment_binary_data_integrity() {
    let data = vec![0xFF, 0x00, 0xAA, 0x55];
    let attachment = Attachment {
        filename: "binary.bin".to_string(),
        data: data.clone(),
        content_type: Some("application/octet-stream".to_string()),
        content_id: None,
        content_location: None,
    };

    assert_eq!(attachment.data, data);
    assert_eq!(attachment.data[0], 0xFF);
    assert_eq!(attachment.data[1], 0x00);
    assert_eq!(attachment.data[2], 0xAA);
    assert_eq!(attachment.data[3], 0x55);
}

#[test]
fn test_attachment_empty_filename_handled() {
    // While ideally this wouldn't happen, test that it can be created
    let attachment = Attachment {
        filename: String::new(),
        data: vec![1, 2, 3],
        content_type: None,
        content_id: None,
        content_location: None,
    };

    assert!(attachment.filename.is_empty());
}

#[test]
fn test_attachment_large_data() {
    let large_data = vec![0u8; 1024 * 1024]; // 1 MB
    let attachment = Attachment {
        filename: "large_file.bin".to_string(),
        data: large_data.clone(),
        content_type: Some("application/octet-stream".to_string()),
        content_id: None,
        content_location: None,
    };

    assert_eq!(attachment.data.len(), 1024 * 1024);
}

#[test]
fn test_attachment_clone() {
    let original = Attachment {
        filename: "test.txt".to_string(),
        data: vec![65, 66, 67], // "ABC"
        content_type: Some("text/plain".to_string()),
        content_id: None,
        content_location: None,
    };

    let cloned = original.clone();

    assert_eq!(original.filename, cloned.filename);
    assert_eq!(original.data, cloned.data);
    assert_eq!(original.content_type, cloned.content_type);
}

// --- US2: Attachment Metadata Normalization & Filename Planning (T014) ---

fn make_attachment(
    filename: &str,
    content_id: Option<&str>,
    content_location: Option<&str>,
) -> Attachment {
    Attachment {
        filename: filename.to_string(),
        data: vec![1, 2, 3],
        content_type: None,
        content_id: content_id.map(String::from),
        content_location: content_location.map(String::from),
    }
}

#[test]
fn test_plan_single_attachment_basic() {
    let attachments = vec![make_attachment("image.png", None, None)];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries.len(), 1);
    assert_eq!(plan.entries[0].resolved_filename, "image.png");
    assert_eq!(plan.entries[0].relative_path, "image.png");
    assert_eq!(plan.entries[0].attachment_index, 0);
}

#[test]
fn test_plan_content_id_normalized_lowercase_stripped() {
    let attachments = vec![make_attachment(
        "img.png",
        Some("<Image001@example.com>"),
        None,
    )];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(
        plan.entries[0].content_id_keys,
        vec!["image001@example.com"]
    );
}

#[test]
fn test_plan_content_id_without_angle_brackets() {
    let attachments = vec![make_attachment(
        "img.png",
        Some("image002@example.com"),
        None,
    )];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(
        plan.entries[0].content_id_keys,
        vec!["image002@example.com"]
    );
}

#[test]
fn test_plan_content_location_normalized_lowercase() {
    let attachments = vec![make_attachment("logo.png", None, Some("Logo.PNG"))];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries[0].content_location_keys, vec!["logo.png"]);
}

#[test]
fn test_plan_empty_content_id_produces_no_keys() {
    let attachments = vec![make_attachment("img.png", Some(""), None)];
    let plan = build_attachment_plan(&attachments);
    assert!(plan.entries[0].content_id_keys.is_empty());
}

#[test]
fn test_plan_empty_content_location_produces_no_keys() {
    let attachments = vec![make_attachment("img.png", None, Some(""))];
    let plan = build_attachment_plan(&attachments);
    assert!(plan.entries[0].content_location_keys.is_empty());
}

#[test]
fn test_plan_filename_collision_resolution() {
    let attachments = vec![
        make_attachment("report.pdf", None, None),
        make_attachment("report.pdf", None, None),
    ];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries[0].resolved_filename, "report.pdf");
    assert_eq!(plan.entries[1].resolved_filename, "report_2.pdf");
}

#[test]
fn test_plan_filename_collision_no_extension() {
    let attachments = vec![
        make_attachment("README", None, None),
        make_attachment("README", None, None),
    ];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries[0].resolved_filename, "README");
    assert_eq!(plan.entries[1].resolved_filename, "README_2");
}

#[test]
fn test_plan_multiple_attachments_mixed_metadata() {
    let attachments = vec![
        make_attachment("header.png", Some("<header@mail>"), None),
        make_attachment("footer.png", None, Some("Footer.png")),
        make_attachment("data.csv", None, None),
    ];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries.len(), 3);
    assert_eq!(plan.entries[0].content_id_keys, vec!["header@mail"]);
    assert!(plan.entries[0].content_location_keys.is_empty());
    assert!(plan.entries[1].content_id_keys.is_empty());
    assert_eq!(plan.entries[1].content_location_keys, vec!["footer.png"]);
    assert!(plan.entries[2].content_id_keys.is_empty());
    assert!(plan.entries[2].content_location_keys.is_empty());
}

#[test]
fn test_plan_preserves_attachment_index() {
    let attachments = vec![
        make_attachment("a.txt", None, None),
        make_attachment("b.txt", None, None),
        make_attachment("c.txt", None, None),
    ];
    let plan = build_attachment_plan(&attachments);
    for (i, entry) in plan.entries.iter().enumerate() {
        assert_eq!(entry.attachment_index, i);
    }
}

#[test]
fn test_plan_empty_attachments() {
    let plan = build_attachment_plan(&[]);
    assert!(plan.entries.is_empty());
}

// --- US3: Disabled-Attachments and Unmatched Cases (T022) ---

use pst_cli::export::html::rewrite_inline_references;

#[test]
fn test_rewrite_with_empty_plan_preserves_all_references() {
    // When attachments are disabled, no plan entries exist → all refs unchanged
    let html = r#"<img src="cid:image@mail"><a href="https://example.com">Link</a>"#;
    let result = rewrite_inline_references(html, &[]);
    assert_eq!(result, html, "Empty plan must return original HTML");
}

#[test]
fn test_plan_attachment_without_inline_metadata() {
    // Attachments with no content_id or content_location should still appear in plan
    // but with empty key lists
    let attachments = vec![make_attachment("report.docx", None, None)];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries.len(), 1);
    assert!(plan.entries[0].content_id_keys.is_empty());
    assert!(plan.entries[0].content_location_keys.is_empty());
    assert_eq!(plan.entries[0].resolved_filename, "report.docx");
}

#[test]
fn test_plan_mix_of_inline_and_regular_attachments() {
    // Some attachments have metadata (inline), some don't (regular)
    let attachments = vec![
        make_attachment("logo.png", Some("<logo@mail>"), None),
        make_attachment("report.pdf", None, None),
        make_attachment("banner.jpg", None, Some("Banner.jpg")),
    ];
    let plan = build_attachment_plan(&attachments);
    assert_eq!(plan.entries.len(), 3);
    // Inline attachment with cid
    assert_eq!(plan.entries[0].content_id_keys, vec!["logo@mail"]);
    // Regular attachment - no keys for lookup
    assert!(plan.entries[1].content_id_keys.is_empty());
    assert!(plan.entries[1].content_location_keys.is_empty());
    // Inline attachment with content-location
    assert_eq!(plan.entries[2].content_location_keys, vec!["banner.jpg"]);
}

#[test]
fn test_rewrite_does_not_crash_on_no_match() {
    // All cid: references in HTML, but plan entries have different keys
    let html = r#"<img src="cid:a@mail"><img src="cid:b@mail">"#;
    let entries = vec![pst_cli::export::exporter::AttachmentExportPlanEntry {
        attachment_index: 0,
        resolved_filename: "x.png".to_string(),
        relative_path: "x.png".to_string(),
        content_id_keys: vec!["c@mail".to_string()],
        content_location_keys: vec![],
    }];
    let result = rewrite_inline_references(html, &entries);
    assert!(
        result.contains("cid:a@mail"),
        "Unmatched preserved: {result}"
    );
    assert!(
        result.contains("cid:b@mail"),
        "Unmatched preserved: {result}"
    );
}
