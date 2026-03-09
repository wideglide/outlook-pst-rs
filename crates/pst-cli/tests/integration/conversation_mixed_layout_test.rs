use pst_cli::export::conversation::{assign_conversation_folders, ConversationCandidate};

#[test]
fn mixed_dataset_only_groups_multi_member_threads() {
    let candidates = vec![
        ConversationCandidate {
            sequence_number: 1,
            conversation_id: Some(b"grouped-id------".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 2,
            conversation_id: Some(b"grouped-id------".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 3,
            conversation_id: Some(b"singleton-id----".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 4,
            conversation_id: None,
            conversation_index: None,
        },
    ];

    let assignments = assign_conversation_folders(&candidates);

    assert_eq!(assignments.get(&1), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&2), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&3), None);
    assert_eq!(assignments.get(&4), None);
}
