//! Conversation grouping helpers for export path assignment.

use std::collections::HashMap;

/// Canonical conversation grouping key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversationKey {
    /// Key derived from `PidTagConversationId` (16 bytes).
    ConversationId([u8; 16]),
    /// Key derived from bytes 6-21 of `PidTagConversationIndex` (16 bytes).
    ConversationIndexBytes([u8; 16]),
}

/// Staged key extraction payload used to compute folder assignments.
#[derive(Debug, Clone)]
pub struct ConversationCandidate {
    /// Export sequence assigned to this message.
    pub sequence_number: u32,
    /// Canonical conversation id if available (16 bytes).
    pub conversation_id: Option<Vec<u8>>,
    /// Raw conversation index bytes for primary grouping (must be at least 22 bytes).
    pub conversation_index: Option<Vec<u8>>,
}

/// Resolve a deterministic `sequence -> conv_#####` map for multi-message groups.
#[must_use]
pub fn assign_conversation_folders(candidates: &[ConversationCandidate]) -> HashMap<u32, String> {
    let mut grouped: HashMap<ConversationKey, Vec<u32>> = HashMap::new();

    for candidate in candidates {
        if let Some(key) = derive_conversation_key(
            candidate.conversation_id.as_deref(),
            candidate.conversation_index.as_deref(),
        ) {
            grouped
                .entry(key)
                .or_default()
                .push(candidate.sequence_number);
        }
    }

    // Only multi-message groups get conversation folders.
    let mut eligible: Vec<Vec<u32>> = grouped
        .into_values()
        .filter(|members| members.len() > 1)
        .collect();

    // Deterministic ordering by the group minimum sequence number.
    eligible.sort_by_key(|members| members.iter().copied().min().unwrap_or(u32::MAX));

    let mut assignments = HashMap::new();
    for (index, members) in eligible.into_iter().enumerate() {
        let folder = format!("conv_{:05}", index + 1);
        for seq in members {
            assignments.insert(seq, folder.clone());
        }
    }

    assignments
}

/// Derive conversation key with precedence:
/// 1) Bytes 6-21 of `PidTagConversationIndex` (16 bytes, requires >= 22 total bytes).
/// 2) `PidTagConversationId` (16 bytes).
/// 3) None
#[must_use]
pub fn derive_conversation_key(
    conversation_id: Option<&[u8]>,
    conversation_index: Option<&[u8]>,
) -> Option<ConversationKey> {
    // Primary: extract bytes 6-21 from ConversationIndex if available.
    if let Some(index) = conversation_index {
        if index.len() >= 22 {
            let mut key_bytes = [0_u8; 16];
            key_bytes.copy_from_slice(&index[6..22]);
            return Some(ConversationKey::ConversationIndexBytes(key_bytes));
        }
    }

    // Alternate: Use ConversationId if available and is exactly 16 bytes.
    if let Some(id) = conversation_id {
        if id.len() == 16 {
            let mut key_bytes = [0_u8; 16];
            key_bytes.copy_from_slice(id);
            return Some(ConversationKey::ConversationId(key_bytes));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_conversation_index_first() {
        let conv_id = vec![1_u8; 16];
        let mut conv_index = vec![0_u8; 22];
        conv_index[6..22].fill(7);
        let key = derive_conversation_key(Some(&conv_id), Some(&conv_index));
        assert_eq!(
            key,
            Some(ConversationKey::ConversationIndexBytes([7_u8; 16]))
        );
    }

    #[test]
    fn derives_fallback_index_bytes_when_id_absent() {
        let conv_index = vec![7_u8; 22];
        let key = derive_conversation_key(None, Some(&conv_index));
        // Bytes 6-21 (16 bytes total) should all be 7.
        assert_eq!(
            key,
            Some(ConversationKey::ConversationIndexBytes([7_u8; 16]))
        );
    }

    #[test]
    fn skips_short_fallback_index() {
        assert_eq!(derive_conversation_key(None, Some(&[9_u8; 21])), None);
    }

    #[test]
    fn rejects_wrong_size_conversation_id() {
        let conv_id_short = vec![1_u8; 15];
        let conv_id_long = vec![1_u8; 17];
        assert_eq!(derive_conversation_key(Some(&conv_id_short), None), None);
        assert_eq!(derive_conversation_key(Some(&conv_id_long), None), None);
    }

    #[test]
    fn extract_correct_offset_from_conversation_index() {
        // ConversationIndex with distinct bytes to verify correct offset extraction.
        let mut conv_index = vec![0_u8; 22];
        // Set bytes 0-5 to different values (reserved/timestamp).
        for (offset, byte) in conv_index.iter_mut().take(6).enumerate() {
            *byte = u8::try_from(offset).expect("offset in range") + 100;
        }
        // Set bytes 6-21 to consistent values for verification.
        for byte in conv_index.iter_mut().take(22).skip(6) {
            *byte = 42;
        }
        let key = derive_conversation_key(None, Some(&conv_index));
        assert_eq!(
            key,
            Some(ConversationKey::ConversationIndexBytes([42_u8; 16]))
        );
    }

    #[test]
    fn assigns_folders_to_multi_message_groups_only() {
        let mut candidates = vec![
            ConversationCandidate {
                sequence_number: 10,
                conversation_id: Some(vec![1_u8; 16]),
                conversation_index: None,
            },
            ConversationCandidate {
                sequence_number: 20,
                conversation_id: Some(vec![2_u8; 16]),
                conversation_index: None,
            },
            ConversationCandidate {
                sequence_number: 11,
                conversation_id: Some(vec![1_u8; 16]),
                conversation_index: None,
            },
            ConversationCandidate {
                sequence_number: 30,
                conversation_id: Some(vec![3_u8; 16]),
                conversation_index: None,
            },
        ];

        // Keep deterministic regardless of input candidate order.
        candidates.reverse();

        let assignments = assign_conversation_folders(&candidates);

        assert_eq!(assignments.get(&10), Some(&"conv_00001".to_string()));
        assert_eq!(assignments.get(&11), Some(&"conv_00001".to_string()));
        assert_eq!(assignments.get(&20), None);
        assert_eq!(assignments.get(&30), None);
    }
}
