use pst_cli::export::exporter::{Attachment, MessageData, MessageExporter};
use tempfile::TempDir;

fn sample_message() -> MessageData {
    MessageData {
        subject: "Conversation message".to_string(),
        from: "sender@example.com".to_string(),
        to: vec!["recipient@example.com".to_string()],
        cc: vec![],
        bcc: vec![],
        date: "2026-03-07T00:00:00+00:00".to_string(),
        message_id: Some("id-1".to_string()),
        body_html: Some("<p>Hello</p>".to_string()),
        body_rtf: None,
        body_plain: None,
        folder_path: "Inbox".to_string(),
        attachments: Vec::<Attachment>::new(),
        headers: None,
        size_bytes: Some(128),
        flags: vec![],
        is_draft: false,
        conversation_id: Some(b"thread1-id------".to_vec()),
        conversation_index: None,
    }
}

#[test]
fn multi_message_layout_uses_shared_conversation_folder() {
    let temp = TempDir::new().unwrap();
    let exporter = MessageExporter::new(temp.path().to_path_buf());
    let message = sample_message();

    exporter
        .export_message(&message, 1, false, Some("conv_00001"))
        .unwrap();
    exporter
        .export_message(&message, 2, false, Some("conv_00001"))
        .unwrap();

    assert!(temp
        .path()
        .join("conv_00001")
        .join("00001")
        .join("message.html")
        .exists());
    assert!(temp
        .path()
        .join("conv_00001")
        .join("00002")
        .join("message.html")
        .exists());
}
