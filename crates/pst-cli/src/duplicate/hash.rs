//! Content hash generation for duplicate detection fallback

use sha2::{Digest, Sha256};

/// Generate SHA-256 content hash from message fields
/// Used as fallback identifier when Message-ID is missing
pub fn generate_content_hash(
    subject: Option<&str>,
    date: Option<&str>,
    from: Option<&str>,
    body: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();

    // Concatenate fields (use empty string if None)
    hasher.update(subject.unwrap_or("").as_bytes());
    hasher.update(date.unwrap_or("").as_bytes());
    hasher.update(from.unwrap_or("").as_bytes());
    hasher.update(body.unwrap_or("").as_bytes());

    // Return hex-encoded hash
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_consistency() {
        let subject = Some("Test Subject");
        let date = Some("2024-01-15");
        let from = Some("sender@example.com");
        let body = Some("Message body");

        let hash1 = generate_content_hash(subject, date, from, body);
        let hash2 = generate_content_hash(subject, date, from, body);

        // Same input produces same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_content_hash_different_messages() {
        let hash1 = generate_content_hash(
            Some("Subject 1"),
            Some("2024-01-15"),
            Some("sender@example.com"),
            Some("Body 1"),
        );

        let hash2 = generate_content_hash(
            Some("Subject 2"),
            Some("2024-01-15"),
            Some("sender@example.com"),
            Some("Body 1"),
        );

        // Different messages produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_none_fields() {
        let hash = generate_content_hash(None, None, None, None);
        
        // Should not panic, should produce valid hash
        assert_eq!(hash.len(), 64);
    }
}
