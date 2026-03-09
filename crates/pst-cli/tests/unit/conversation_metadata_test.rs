use pst_cli::export::conversation::{assign_conversation_folders, ConversationCandidate};
use pst_cli::export::exporter::{Attachment, MessageData};
use pst_cli::export::metadata::format_metadata;

fn message_with_conversation(conversation_id: Option<&[u8]>) -> MessageData {
    MessageData {
        subject: "Subject".to_string(),
        from: "from@example.com".to_string(),
        to: vec!["to@example.com".to_string()],
        cc: vec![],
        bcc: vec![],
        date: "2026-03-07T00:00:00+00:00".to_string(),
        message_id: Some("id-1".to_string()),
        body_html: Some("<p>Body</p>".to_string()),
        body_rtf: None,
        body_plain: None,
        folder_path: "Inbox".to_string(),
        attachments: Vec::<Attachment>::new(),
        headers: None,
        size_bytes: Some(42),
        flags: vec![],
        is_draft: false,
        conversation_id: conversation_id.map(<[u8]>::to_vec),
        conversation_index: None,
    }
}

#[test]
fn singleton_group_is_not_assigned_conversation_folder() {
    let candidates = vec![ConversationCandidate {
        sequence_number: 1,
        conversation_id: Some(b"single-------123".to_vec()),
        conversation_index: None,
    }];

    let assignments = assign_conversation_folders(&candidates);
    assert_eq!(assignments.get(&1), None);
}

#[test]
fn metadata_includes_conversation_id_only_when_present() {
    let with_id = message_with_conversation(Some(b"abc123abc123abc1"));
    let without_id = message_with_conversation(None);

    let with_content = format_metadata(&with_id, &[], &[]);
    let without_content = format_metadata(&without_id, &[], &[]);

    assert!(with_content.contains("ConversationId:"));
    assert!(!without_content.contains("ConversationId:"));
}
