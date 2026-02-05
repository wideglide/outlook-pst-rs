//! Keyword matching logic

use std::collections::HashSet;

/// Keyword matcher for case-insensitive keyword search
#[derive(Debug)]
pub struct KeywordMatcher {
    /// Normalized keywords (lowercase)
    keywords: HashSet<String>,
}

impl KeywordMatcher {
    /// Create a new keyword matcher
    pub fn new(keywords: Vec<String>) -> Self {
        let keywords = keywords.into_iter().collect();
        Self { keywords }
    }

    /// Check if text contains any keywords (case-insensitive)
    /// Returns set of matched keywords
    pub fn find_matches(&self, text: &str) -> HashSet<String> {
        let text_lower = text.to_lowercase();
        
        self.keywords
            .iter()
            .filter(|kw| text_lower.contains(kw.as_str()))
            .cloned()
            .collect()
    }

    /// Search message subject and body for keywords
    pub fn search_message(
        &self,
        subject: Option<&str>,
        body: Option<&str>,
    ) -> HashSet<String> {
        let mut matches = HashSet::new();

        if let Some(subj) = subject {
            matches.extend(self.find_matches(subj));
        }

        if let Some(body_text) = body {
            matches.extend(self.find_matches(body_text));
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_insensitive_matching() {
        let matcher = KeywordMatcher::new(vec!["confidential".to_string(), "merger".to_string()]);

        let matches = matcher.find_matches("This is a CONFIDENTIAL document about the merger.");
        assert_eq!(matches.len(), 2);
        assert!(matches.contains("confidential"));
        assert!(matches.contains("merger"));
    }

    #[test]
    fn test_search_message() {
        let matcher = KeywordMatcher::new(vec!["urgent".to_string(), "action".to_string()]);

        let matches = matcher.search_message(
            Some("URGENT: Action Required"),
            Some("Please take action immediately."),
        );

        assert_eq!(matches.len(), 2);
        assert!(matches.contains("urgent"));
        assert!(matches.contains("action"));
    }

    #[test]
    fn test_no_matches() {
        let matcher = KeywordMatcher::new(vec!["confidential".to_string()]);

        let matches = matcher.find_matches("This is a regular document.");
        assert_eq!(matches.len(), 0);
    }
}
