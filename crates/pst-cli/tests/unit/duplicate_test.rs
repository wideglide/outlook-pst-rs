//! Unit tests for duplicate detection functionality
//!
//! Tests T042-T043: Message-ID extraction, content hash generation, duplicate detection

use pst_cli::duplicate::{DuplicateTracker, extract_message_id, generate_message_identifier};
use pst_cli::duplicate::hash::generate_content_hash;
use pst_cli::export::exporter::MessageData;

#[test]
fn test_message_id_extraction_with_brackets() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("<abc123@example.com>".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let extracted = extract_message_id(&message);
    // Should remove angle brackets and lowercase
    assert_eq!(extracted, "abc123@example.com");
}

#[test]
fn test_message_id_extraction_without_brackets() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("XYZ789@example.com".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let extracted = extract_message_id(&message);
    // Should lowercase
    assert_eq!(extracted, "xyz789@example.com");
}

#[test]
fn test_message_id_extraction_with_whitespace() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("  <  test@example.com  >  ".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let extracted = extract_message_id(&message);
    // Should trim whitespace and remove brackets
    assert_eq!(extracted, "test@example.com");
}

#[test]
fn test_message_id_extraction_none() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: None,
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let extracted = extract_message_id(&message);
    // Should return empty string
    assert_eq!(extracted, "");
}

#[test]
fn test_content_hash_consistency() {
    let hash1 = generate_content_hash(
        Some("Subject"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Message body"),
    );

    let hash2 = generate_content_hash(
        Some("Subject"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Message body"),
    );

    // Same input should produce same hash
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // SHA-256 hex string
}

#[test]
fn test_content_hash_different_subject() {
    let hash1 = generate_content_hash(
        Some("Subject 1"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Body"),
    );

    let hash2 = generate_content_hash(
        Some("Subject 2"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Body"),
    );

    // Different subject should produce different hash
    assert_ne!(hash1, hash2);
}

#[test]
fn test_content_hash_different_body() {
    let hash1 = generate_content_hash(
        Some("Subject"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Body 1"),
    );

    let hash2 = generate_content_hash(
        Some("Subject"),
        Some("2026-02-11"),
        Some("sender@example.com"),
        Some("Body 2"),
    );

    // Different body should produce different hash
    assert_ne!(hash1, hash2);
}

#[test]
fn test_generate_identifier_with_message_id() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("<test@example.com>".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let identifier = generate_message_identifier(&message);
    
    // Should use Message-ID
    assert!(identifier.starts_with("msgid:"));
    assert!(identifier.contains("test@example.com"));
}

#[test]
fn test_generate_identifier_without_message_id() {
    let message = MessageData {
        subject: "Test Subject".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: None,
        body_html: None,
        body_rtf: None,
        body_plain: Some("Test body content".to_string()),        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let identifier = generate_message_identifier(&message);
    
    // Should fallback to hash
    assert!(identifier.starts_with("hash:"));
    assert_eq!(identifier.len(), 5 + 64); // "hash:" + 64 hex chars
}

#[test]
fn test_duplicate_tracker_first_message() {
    let mut tracker = DuplicateTracker::new();
    
    let (is_dup, first_seq) = tracker.check_and_record("msg1", 1);
    
    assert!(!is_dup, "First occurrence should not be duplicate");
    assert_eq!(first_seq, None);
}

#[test]
fn test_duplicate_tracker_duplicate_detection() {
    let mut tracker = DuplicateTracker::new();
    
    tracker.check_and_record("msg1", 1);
    tracker.check_and_record("msg2", 2);
    
    let (is_dup, first_seq) = tracker.check_and_record("msg1", 3);
    
    assert!(is_dup, "Should detect duplicate");
    assert_eq!(first_seq, Some(1), "Should return first sequence number");
}

#[test]
fn test_duplicate_tracker_multiple_duplicates() {
    let mut tracker = DuplicateTracker::new();
    
    tracker.check_and_record("msgA", 1);
    let (is_dup2, _) = tracker.check_and_record("msgA", 2);
    let (is_dup3, first) = tracker.check_and_record("msgA", 3);
    
    assert!(is_dup2, "Second occurrence should be duplicate");
    assert!(is_dup3, "Third occurrence should be duplicate");
    assert_eq!(first, Some(1), "All duplicates should reference first occurrence");
}

#[test]
fn test_duplicate_tracker_unique_count() {
    let mut tracker = DuplicateTracker::new();
    
    tracker.check_and_record("msg1", 1);
    tracker.check_and_record("msg2", 2);
    tracker.check_and_record("msg1", 3); // duplicate
    tracker.check_and_record("msg3", 4);
    
    assert_eq!(tracker.unique_count(), 3, "Should count only unique messages");
}

#[test]
fn test_identifier_case_insensitivity() {
    let message1 = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("<ABC@example.com>".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let message2 = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("<abc@example.com>".to_string()),
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let id1 = generate_message_identifier(&message1);
    let id2 = generate_message_identifier(&message2);
    
    // Should be same identifier (case-insensitive)
    assert_eq!(id1, id2);
}

#[test]
fn test_fallback_to_hash_for_empty_message_id() {
    let message = MessageData {
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: vec![],
        cc: vec![],
        bcc: vec![],
        date: "2026-02-11".to_string(),
        message_id: Some("".to_string()), // Empty string
        body_html: None,
        body_rtf: None,
        body_plain: Some("Body".to_string()),
        attachments: vec![],
        headers: None,
        size_bytes: None,
        flags: vec![],
        is_draft: false,
        folder_path: "Inbox".to_string(),
    };

    let identifier = generate_message_identifier(&message);
    
    // Should fallback to hash when Message-ID is empty
    assert!(identifier.starts_with("hash:"));
}
