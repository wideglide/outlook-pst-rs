use pst_cli::export::conversation::{assign_conversation_folders, ConversationCandidate};

#[test]
fn fallback_index_groups_messages_when_prefix_matches() {
    let mut key_a = vec![10_u8; 30];
    let key_b = vec![10_u8; 22];
    key_a[22] = 99;

    let candidates = vec![
        ConversationCandidate {
            sequence_number: 1,
            conversation_id: None,
            conversation_index: Some(key_a),
        },
        ConversationCandidate {
            sequence_number: 2,
            conversation_id: None,
            conversation_index: Some(key_b),
        },
    ];

    let assignments = assign_conversation_folders(&candidates);

    assert_eq!(assignments.get(&1), Some(&"conv_00001".to_string()));
    assert_eq!(assignments.get(&2), Some(&"conv_00001".to_string()));
}

#[test]
fn short_fallback_index_stays_ungrouped() {
    let candidates = vec![
        ConversationCandidate {
            sequence_number: 1,
            conversation_id: None,
            conversation_index: Some(vec![1_u8; 21]),
        },
        ConversationCandidate {
            sequence_number: 2,
            conversation_id: None,
            conversation_index: Some(vec![1_u8; 21]),
        },
    ];

    let assignments = assign_conversation_folders(&candidates);

    assert_eq!(assignments.get(&1), None);
    assert_eq!(assignments.get(&2), None);
}
