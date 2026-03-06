//! Duplicate detection
//!
//! Detects duplicate messages using Message-ID header with fallback to content hash.
//! Supports tracking duplicates across multiple PST files for comprehensive deduplication.

pub mod hash;

use std::collections::HashMap;
use crate::export::exporter::MessageData;

/// Duplicate tracker for messages across all PST files
#[derive(Debug, Default)]
pub struct DuplicateTracker {
    /// Map of message identifiers (Message-ID or content hash) to sequence number
    seen: HashMap<String, u32>,
}

impl DuplicateTracker {
    /// Create a new duplicate tracker
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a message is a duplicate and record it
    /// Returns (`is_duplicate`, `first_sequence_number`)
    pub fn check_and_record(&mut self, identifier: &str, sequence: u32) -> (bool, Option<u32>) {
        if let Some(&first_seq) = self.seen.get(identifier) {
            // Duplicate found
            (true, Some(first_seq))
        } else {
            // First occurrence
            self.seen.insert(identifier.to_string(), sequence);
            (false, None)
        }
    }

    /// Get total number of unique messages seen
    #[must_use] 
    pub fn unique_count(&self) -> usize {
        self.seen.len()
    }

    /// Get total number of duplicates found
    #[must_use] 
    pub fn duplicate_count(&self) -> usize {
        // This is tracked by the caller since we don't store duplicate count internally
        // This method exists for API completeness
        0
    }
}

/// Extract Message-ID from message headers
/// Returns normalized Message-ID or empty string if not present
#[must_use] 
pub fn extract_message_id(message: &MessageData) -> String {
    message.message_id
        .as_ref()
        .map(|id| normalize_message_id(id))
        .unwrap_or_default()
}

/// Normalize Message-ID for consistent comparison
/// Removes angle brackets, trims whitespace, converts to lowercase
fn normalize_message_id(message_id: &str) -> String {
    message_id
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim()  // Trim again after removing brackets
        .to_lowercase()
}

/// Generate unique identifier for a message
/// Uses Message-ID if available, otherwise generates content hash
#[must_use] 
pub fn generate_message_identifier(message: &MessageData) -> String {
    let message_id = extract_message_id(message);
    
    if message_id.is_empty() {
        // Fallback to content hash
        let body_text = message.body_plain.as_deref()
            .or(message.body_html.as_deref())
            .unwrap_or("");
        
        let hash = hash::generate_content_hash(
            Some(&message.subject),
            Some(&message.date),
            Some(&message.from),
            Some(body_text),
        );
        
        format!("hash:{hash}")
    } else {
        // Use Message-ID as identifier
        format!("msgid:{message_id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_detection() {
        let mut tracker = DuplicateTracker::new();

        // First message with ID "abc123"
        let (is_dup, first_seq) = tracker.check_and_record("abc123", 1);
        assert!(!is_dup);
        assert_eq!(first_seq, None);

        // Different message with ID "xyz789"
        let (is_dup, first_seq) = tracker.check_and_record("xyz789", 2);
        assert!(!is_dup);
        assert_eq!(first_seq, None);

        // Duplicate of first message
        let (is_dup, first_seq) = tracker.check_and_record("abc123", 3);
        assert!(is_dup);
        assert_eq!(first_seq, Some(1));

        assert_eq!(tracker.unique_count(), 2);
    }
}
