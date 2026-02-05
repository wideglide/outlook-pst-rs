//! Email participant matching logic

use crate::EmailAddress;
use std::collections::HashSet;

/// Email matcher for case-insensitive email address search
#[derive(Debug)]
pub struct EmailMatcher {
    /// Normalized email addresses (lowercase)
    emails: HashSet<String>,
}

impl EmailMatcher {
    /// Create a new email matcher
    pub fn new(emails: Vec<String>) -> Self {
        let emails = emails.into_iter().collect();
        Self { emails }
    }

    /// Extract email address from EmailAddress struct (normalized)
    fn normalize_email(addr: &EmailAddress) -> String {
        addr.email_address.to_lowercase()
    }

    /// Check if email list contains any target addresses
    /// Returns set of matched email addresses
    pub fn find_matches(&self, addresses: &[EmailAddress]) -> HashSet<String> {
        addresses
            .iter()
            .map(Self::normalize_email)
            .filter(|email| self.emails.contains(email))
            .collect()
    }

    /// Search message From, To, CC, BCC fields for email addresses
    pub fn search_message(
        &self,
        from: Option<&EmailAddress>,
        to: &[EmailAddress],
        cc: &[EmailAddress],
        bcc: &[EmailAddress],
    ) -> HashSet<String> {
        let mut matches = HashSet::new();

        if let Some(from_addr) = from {
            let normalized = Self::normalize_email(from_addr);
            if self.emails.contains(&normalized) {
                matches.insert(normalized);
            }
        }

        matches.extend(self.find_matches(to));
        matches.extend(self.find_matches(cc));
        matches.extend(self.find_matches(bcc));

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_matching() {
        let matcher = EmailMatcher::new(vec![
            "john@example.com".to_string(),
            "jane@example.com".to_string(),
        ]);

        let addresses = vec![
            EmailAddress::new("JOHN@EXAMPLE.COM"),
            EmailAddress::new("other@example.com"),
        ];

        let matches = matcher.find_matches(&addresses);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("john@example.com"));
    }

    #[test]
    fn test_search_message() {
        let matcher = EmailMatcher::new(vec!["target@example.com".to_string()]);

        let from = EmailAddress::new("sender@example.com");
        let to = vec![
            EmailAddress::new("TARGET@EXAMPLE.COM"),
            EmailAddress::new("other@example.com"),
        ];
        let cc = vec![];
        let bcc = vec![];

        let matches = matcher.search_message(Some(&from), &to, &cc, &bcc);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains("target@example.com"));
    }

    #[test]
    fn test_no_matches() {
        let matcher = EmailMatcher::new(vec!["target@example.com".to_string()]);

        let addresses = vec![
            EmailAddress::new("other1@example.com"),
            EmailAddress::new("other2@example.com"),
        ];

        let matches = matcher.find_matches(&addresses);
        assert_eq!(matches.len(), 0);
    }
}
