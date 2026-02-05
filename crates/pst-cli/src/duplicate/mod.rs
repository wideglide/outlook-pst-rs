//! Duplicate detection
//!
//! NOTE: This module structure currently uses HashMap directly.
//! In the implementation phase, this should import std::collections::HashMap properly.

pub mod hash;

use std::collections::HashMap;

/// Duplicate tracker for messages across all PST files
#[derive(Debug, Default)]
pub struct DuplicateTracker {
    /// Map of message identifiers (Message-ID or content hash) to sequence number
    seen: HashMap<String, u32>,
}

impl DuplicateTracker {
    /// Create a new duplicate tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a message is a duplicate and record it
    /// Returns (is_duplicate, first_sequence_number)
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
    pub fn unique_count(&self) -> usize {
        self.seen.len()
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
