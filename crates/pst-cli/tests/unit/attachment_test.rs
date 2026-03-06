//! Unit tests for attachment export functionality

use pst_cli::export::exporter::Attachment;

#[test]
fn test_attachment_creation() {
    let attachment = Attachment {
        filename: "document.pdf".to_string(),
        data: vec![0x25, 0x50, 0x44, 0x46], // PDF header: %PDF
        content_type: Some("application/pdf".to_string()),
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
        content_type: Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
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
    };

    assert_eq!(attachment.data.len(), 1024 * 1024);
}

#[test]
fn test_attachment_clone() {
    let original = Attachment {
        filename: "test.txt".to_string(),
        data: vec![65, 66, 67], // "ABC"
        content_type: Some("text/plain".to_string()),
    };

    let cloned = original.clone();

    assert_eq!(original.filename, cloned.filename);
    assert_eq!(original.data, cloned.data);
    assert_eq!(original.content_type, cloned.content_type);
}
