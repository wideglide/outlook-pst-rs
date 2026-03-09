//! Email participant matching logic
//!
//! Performs case-insensitive email address matching across message participants
//! (From, To, CC, BCC). Matches on the email address portion only, ignoring
//! display names. Supports parsing from "Display Name <address>" format.

use std::collections::HashSet;

/// Email matcher for case-insensitive email address search
#[derive(Debug)]
pub struct EmailMatcher {
    /// Normalized target email addresses (lowercase, trimmed)
    emails: HashSet<String>,
}

impl EmailMatcher {
    /// Create a new email matcher from a vector of email address strings.
    /// Normalizes to lowercase, trims whitespace, extracts address from
    /// "Display Name <addr>" format, and deduplicates.
    #[must_use]
    pub fn new(emails: Vec<String>) -> Self {
        let emails = emails
            .into_iter()
            .map(|e| extract_email_address(&e).to_lowercase())
            .filter(|e| !e.is_empty())
            .collect();
        Self { emails }
    }

    /// Create a new email matcher from a comma-separated string.
    /// Handles parsing, trimming, lowercasing, extraction, and deduplication.
    #[must_use]
    pub fn from_string(emails_str: &str) -> Self {
        let emails = emails_str
            .split(',')
            .map(|e| extract_email_address(e.trim()).to_lowercase())
            .filter(|e| !e.is_empty())
            .collect::<HashSet<_>>();
        Self { emails }
    }

    /// Get the sorted list of target email addresses
    #[must_use]
    pub fn emails(&self) -> Vec<String> {
        let mut emails: Vec<_> = self.emails.iter().cloned().collect();
        emails.sort();
        emails
    }

    /// Check if any of the provided address strings match target emails.
    /// Each address string is normalized (extract address, lowercase) before comparison.
    /// Returns set of matched email addresses (from the target list).
    #[must_use]
    pub fn find_matches_str(&self, addresses: &[String]) -> HashSet<String> {
        addresses
            .iter()
            .map(|addr| extract_email_address(addr).to_lowercase())
            .filter(|email| self.emails.contains(email))
            .collect()
    }

    /// Search message From, To, CC, BCC string fields for target email addresses.
    /// Returns set of matched email addresses (presence only, deduplicated).
    #[must_use]
    pub fn search_message(
        &self,
        from: &str,
        to: &[String],
        cc: &[String],
        bcc: &[String],
    ) -> HashSet<String> {
        let mut matches = HashSet::new();

        // Check From field
        let from_normalized = extract_email_address(from).to_lowercase();
        if self.emails.contains(&from_normalized) {
            matches.insert(from_normalized);
        }

        // Check To, CC, BCC fields
        matches.extend(self.find_matches_str(to));
        matches.extend(self.find_matches_str(cc));
        matches.extend(self.find_matches_str(bcc));

        matches
    }

    /// Get count of matched emails
    #[must_use]
    pub fn match_count(matches: &HashSet<String>) -> usize {
        matches.len()
    }
}

/// Extract the email address from a string that may be in
/// "Display Name <address@domain>" format or just "address@domain".
///
/// Returns the address portion only, trimmed. If no angle brackets found,
/// returns the entire string trimmed.
#[must_use]
pub fn extract_email_address(input: &str) -> String {
    let trimmed = input.trim();
    if let Some(start) = trimmed.rfind('<') {
        if let Some(end) = trimmed.rfind('>') {
            if start < end {
                return trimmed[start + 1..end].trim().to_string();
            }
        }
    }
    // No angle brackets - return as-is (trimmed)
    trimmed.to_string()
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;

    // --- extract_email_address tests ---

    #[test]
    fn test_extract_plain_address() {
        assert_eq!(
            extract_email_address("john@example.com"),
            "john@example.com"
        );
    }

    #[test]
    fn test_extract_from_display_name_format() {
        assert_eq!(
            extract_email_address("John Doe <john@example.com>"),
            "john@example.com"
        );
    }

    #[test]
    fn test_extract_with_whitespace() {
        assert_eq!(
            extract_email_address("  Jane Doe  <  jane@example.com  >  "),
            "jane@example.com"
        );
    }

    #[test]
    fn test_extract_address_only_in_brackets() {
        assert_eq!(
            extract_email_address("<user@domain.org>"),
            "user@domain.org"
        );
    }

    #[test]
    fn test_extract_empty_string() {
        assert_eq!(extract_email_address(""), "");
    }

    #[test]
    fn test_extract_no_at_sign() {
        assert_eq!(extract_email_address("not-an-email"), "not-an-email");
    }

    // --- EmailMatcher::new tests ---

    #[test]
    fn test_new_normalizes_to_lowercase() {
        let matcher = EmailMatcher::new(vec![
            "JOHN@EXAMPLE.COM".to_string(),
            "Jane@Example.Com".to_string(),
        ]);
        let emails = matcher.emails();
        assert!(emails.contains(&"john@example.com".to_string()));
        assert!(emails.contains(&"jane@example.com".to_string()));
    }

    #[test]
    fn test_new_extracts_from_display_name() {
        let matcher = EmailMatcher::new(vec!["John Doe <john@example.com>".to_string()]);
        let emails = matcher.emails();
        assert_eq!(emails, vec!["john@example.com"]);
    }

    #[test]
    fn test_new_deduplicates() {
        let matcher = EmailMatcher::new(vec![
            "john@example.com".to_string(),
            "JOHN@EXAMPLE.COM".to_string(),
            "John Doe <john@example.com>".to_string(),
        ]);
        assert_eq!(matcher.emails().len(), 1);
    }

    #[test]
    fn test_new_filters_empty() {
        let matcher = EmailMatcher::new(vec![
            String::new(),
            "  ".to_string(),
            "valid@example.com".to_string(),
        ]);
        // Empty strings after trim become "", which is filtered out
        // "  " trims to "", also filtered
        assert_eq!(matcher.emails().len(), 1);
    }

    // --- EmailMatcher::from_string tests ---

    #[test]
    fn test_from_string_comma_separated() {
        let matcher = EmailMatcher::from_string("john@example.com, jane@example.com, bob@test.org");
        let emails = matcher.emails();
        assert_eq!(emails.len(), 3);
        assert!(emails.contains(&"john@example.com".to_string()));
        assert!(emails.contains(&"jane@example.com".to_string()));
        assert!(emails.contains(&"bob@test.org".to_string()));
    }

    #[test]
    fn test_from_string_with_display_names() {
        let matcher =
            EmailMatcher::from_string("John <john@example.com>, Jane Doe <jane@example.com>");
        let emails = matcher.emails();
        assert_eq!(emails.len(), 2);
        assert!(emails.contains(&"john@example.com".to_string()));
        assert!(emails.contains(&"jane@example.com".to_string()));
    }

    #[test]
    fn test_from_string_deduplication() {
        let matcher = EmailMatcher::from_string(
            "john@example.com, JOHN@EXAMPLE.COM, John <john@example.com>",
        );
        assert_eq!(matcher.emails().len(), 1);
    }

    #[test]
    fn test_from_string_empty() {
        let matcher = EmailMatcher::from_string("");
        assert_eq!(matcher.emails().len(), 0);
    }

    #[test]
    fn test_from_string_with_empty_entries() {
        let matcher = EmailMatcher::from_string(",  , valid@example.com,  , another@test.com,  ");
        assert_eq!(matcher.emails().len(), 2);
    }

    // --- find_matches_str tests ---

    #[test]
    fn test_find_matches_case_insensitive() {
        let matcher = EmailMatcher::new(vec![
            "john@example.com".to_string(),
            "jane@example.com".to_string(),
        ]);

        let addresses = vec![
            "JOHN@EXAMPLE.COM".to_string(),
            "other@example.com".to_string(),
        ];

        let matches = matcher.find_matches_str(&addresses);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("john@example.com"));
    }

    #[test]
    fn test_find_matches_with_display_names() {
        let matcher = EmailMatcher::new(vec!["john@example.com".to_string()]);

        let addresses = vec!["John Doe <JOHN@EXAMPLE.COM>".to_string()];

        let matches = matcher.find_matches_str(&addresses);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("john@example.com"));
    }

    #[test]
    fn test_find_matches_no_false_positives_from_display_name() {
        // Display name contains "john@example.com" but email address is different
        let matcher = EmailMatcher::new(vec!["john@example.com".to_string()]);

        let addresses = vec!["other@domain.com".to_string()];

        let matches = matcher.find_matches_str(&addresses);
        assert_eq!(matches.len(), 0);
    }

    // --- search_message tests ---

    #[test]
    fn test_search_message_from_match() {
        let matcher = EmailMatcher::new(vec!["sender@example.com".to_string()]);

        let matches = matcher.search_message("Sender Name <sender@example.com>", &[], &[], &[]);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("sender@example.com"));
    }

    #[test]
    fn test_search_message_to_match() {
        let matcher = EmailMatcher::new(vec!["target@example.com".to_string()]);

        let matches = matcher.search_message(
            "sender@other.com",
            &[
                "TARGET@EXAMPLE.COM".to_string(),
                "other@example.com".to_string(),
            ],
            &[],
            &[],
        );
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("target@example.com"));
    }

    #[test]
    fn test_search_message_cc_match() {
        let matcher = EmailMatcher::new(vec!["cc@example.com".to_string()]);

        let matches = matcher.search_message(
            "sender@other.com",
            &[],
            &["CC@EXAMPLE.COM".to_string()],
            &[],
        );
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("cc@example.com"));
    }

    #[test]
    fn test_search_message_bcc_match() {
        let matcher = EmailMatcher::new(vec!["bcc@example.com".to_string()]);

        let matches = matcher.search_message(
            "sender@other.com",
            &[],
            &[],
            &["BCC@example.com".to_string()],
        );
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("bcc@example.com"));
    }

    #[test]
    fn test_search_message_multi_field_match() {
        let matcher = EmailMatcher::new(vec![
            "alice@example.com".to_string(),
            "bob@example.com".to_string(),
            "charlie@example.com".to_string(),
        ]);

        let matches = matcher.search_message(
            "Alice <alice@example.com>",
            &["Bob <bob@example.com>".to_string()],
            &["Charlie <charlie@example.com>".to_string()],
            &[],
        );
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_search_message_deduplication_across_fields() {
        // Same person in From and CC - should only appear once
        let matcher = EmailMatcher::new(vec!["person@example.com".to_string()]);

        let matches = matcher.search_message(
            "person@example.com",
            &["person@example.com".to_string()],
            &["person@example.com".to_string()],
            &["person@example.com".to_string()],
        );
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_search_message_no_matches() {
        let matcher = EmailMatcher::new(vec!["target@example.com".to_string()]);

        let matches = matcher.search_message(
            "other1@example.com",
            &["other2@example.com".to_string()],
            &["other3@example.com".to_string()],
            &[],
        );
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_count() {
        let mut matches = HashSet::new();
        matches.insert("a@example.com".to_string());
        matches.insert("b@example.com".to_string());
        assert_eq!(EmailMatcher::match_count(&matches), 2);
    }
}
