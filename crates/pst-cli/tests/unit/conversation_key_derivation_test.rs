use pst_cli::export::conversation::{derive_conversation_key, ConversationKey};

#[test]
fn derives_fallback_key_from_bytes_6_21() {
    let mut bytes = vec![0_u8; 30];
    for (index, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::try_from(index).expect("index within u8 range");
    }

    let key = derive_conversation_key(None, Some(&bytes));

    // Bytes 6-21 (16 bytes) should come from indices 6-22
    let mut expected = [0_u8; 16];
    expected.copy_from_slice(&bytes[6..22]);
    assert_eq!(key, Some(ConversationKey::ConversationIndexBytes(expected)));
}

#[test]
fn short_conversation_index_produces_no_key() {
    let short = [1_u8; 21];
    assert_eq!(derive_conversation_key(None, Some(&short)), None);
}

#[test]
fn conversation_index_takes_precedence_over_conversation_id() {
    let mut bytes = [0_u8; 22];
    bytes[6..22].fill(2);
    let cid = [1_u8; 16];
    let key = derive_conversation_key(Some(&cid), Some(&bytes));

    assert_eq!(key, Some(ConversationKey::ConversationIndexBytes([2_u8; 16])));
}

#[test]
fn conversation_id_is_used_when_index_is_missing() {
    let cid = [3_u8; 16];

    assert_eq!(
        derive_conversation_key(Some(&cid), None),
        Some(ConversationKey::ConversationId(cid))
    );
}
