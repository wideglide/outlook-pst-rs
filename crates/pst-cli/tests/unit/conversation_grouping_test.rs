use pst_cli::export::conversation::{assign_conversation_folders, ConversationCandidate};

#[test]
fn groups_messages_with_same_conversation_id() {
    let candidates = vec![
        ConversationCandidate {
            sequence_number: 1,
            conversation_id: Some(b"thread-a-id-----".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 2,
            conversation_id: Some(b"thread-a-id-----".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 3,
            conversation_id: Some(b"thread-b-id-----".to_vec()),
            conversation_index: None,
        },
    ];

    let assignments = assign_conversation_folders(&candidates);

    assert_eq!(assignments.get(&1), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&2), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&3), None);
}

#[test]
fn numbering_uses_minimum_group_sequence() {
    let candidates = vec![
        ConversationCandidate {
            sequence_number: 20,
            conversation_id: Some(b"later-id--------".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 21,
            conversation_id: Some(b"later-id--------".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 5,
            conversation_id: Some(b"earlier-id------".to_vec()),
            conversation_index: None,
        },
        ConversationCandidate {
            sequence_number: 6,
            conversation_id: Some(b"earlier-id------".to_vec()),
            conversation_index: None,
        },
    ];

    let assignments = assign_conversation_folders(&candidates);

    assert_eq!(assignments.get(&5), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&6), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&20), Some(&"conv_00002".to_string()));
    assert_eq!(assignments.get(&21), Some(&"conv_00002".to_string()));
}
