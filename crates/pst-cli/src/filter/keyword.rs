//! Keyword matching logic

use std::collections::HashSet;

/// Keyword matcher for case-insensitive keyword search
#[derive(Debug)]
pub struct KeywordMatcher {
    /// Normalized keywords (lowercase)
    keywords: HashSet<String>,
}

impl KeywordMatcher {
    /// Create a new keyword matcher from comma-separated keyword string
    /// Handles parsing, trimming, lowercasing, and deduplication
    #[must_use]
    pub fn from_string(keywords_str: &str) -> Self {
        let keywords = keywords_str
            .split(',')
            .map(|kw| kw.trim().to_lowercase())
            .filter(|kw| !kw.is_empty())
            .collect::<HashSet<_>>();

        Self { keywords }
    }

    /// Create a new keyword matcher from a vector of keywords
    #[must_use]
    pub fn new(keywords: Vec<String>) -> Self {
        let keywords = keywords.into_iter().map(|kw| kw.to_lowercase()).collect();
        Self { keywords }
    }

    /// Get the list of keywords being searched for
    #[must_use]
    pub fn keywords(&self) -> Vec<String> {
        let mut kws: Vec<_> = self.keywords.iter().cloned().collect();
        kws.sort();
        kws
    }

    /// Check if text contains any keywords (case-insensitive)
    /// Returns set of matched keywords
    #[must_use]
    pub fn find_matches(&self, text: &str) -> HashSet<String> {
        let text_lower = text.to_lowercase();

        self.keywords
            .iter()
            .filter(|kw| text_lower.contains(kw.as_str()))
            .cloned()
            .collect()
    }

    /// Search message subject and body for keywords
    /// Returns set of matched keywords (presence only, not count of occurrences)
    #[must_use]
    pub fn search_message(&self, subject: Option<&str>, body: Option<&str>) -> HashSet<String> {
        let mut matches = HashSet::new();

        if let Some(subj) = subject {
            matches.extend(self.find_matches(subj));
        }

        if let Some(body_text) = body {
            matches.extend(self.find_matches(body_text));
        }

        matches
    }

    /// Get count of matched keywords
    #[must_use]
    pub fn match_count(matches: &HashSet<String>) -> usize {
        matches.len()
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
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
    fn test_keyword_parsing_from_string() {
        let matcher = KeywordMatcher::from_string("Confidential, MERGER, Action");

        let keywords = matcher.keywords();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&"confidential".to_string()));
        assert!(keywords.contains(&"merger".to_string()));
        assert!(keywords.contains(&"action".to_string()));
    }

    #[test]
    fn test_keyword_parsing_with_whitespace() {
        let matcher = KeywordMatcher::from_string("  urgent  ,  action  ,  needed  ");

        let keywords = matcher.keywords();
        assert_eq!(keywords.len(), 3);
        assert!(keywords.contains(&"urgent".to_string()));
        assert!(keywords.contains(&"action".to_string()));
        assert!(keywords.contains(&"needed".to_string()));
    }

    #[test]
    fn test_keyword_deduplication() {
        let matcher = KeywordMatcher::from_string("urgent, URGENT, Urgent");

        let keywords = matcher.keywords();
        assert_eq!(keywords.len(), 1);
        assert!(keywords.contains(&"urgent".to_string()));
    }

    #[test]
    fn test_keyword_parsing_empty_string() {
        let matcher = KeywordMatcher::from_string("");

        let keywords = matcher.keywords();
        assert_eq!(keywords.len(), 0);
    }

    #[test]
    fn test_keyword_parsing_empty_entries() {
        let matcher = KeywordMatcher::from_string(",  , urgent,  , action,  ");

        let keywords = matcher.keywords();
        assert_eq!(keywords.len(), 2);
        assert!(keywords.contains(&"urgent".to_string()));
        assert!(keywords.contains(&"action".to_string()));
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
    fn test_search_message_subject_only() {
        let matcher = KeywordMatcher::new(vec!["urgent".to_string(), "action".to_string()]);

        let matches = matcher.search_message(Some("URGENT: Action Required"), None);

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_search_message_body_only() {
        let matcher = KeywordMatcher::new(vec!["urgent".to_string(), "action".to_string()]);

        let matches = matcher.search_message(
            None,
            Some("Please take action immediately. This is urgent!"),
        );

        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_search_message_partial_match() {
        let matcher = KeywordMatcher::new(vec!["act".to_string()]);

        let matches = matcher.search_message(Some("Action required"), None);

        assert_eq!(matches.len(), 1);
        assert!(matches.contains("act"));
    }

    #[test]
    fn test_no_matches() {
        let matcher = KeywordMatcher::new(vec!["confidential".to_string()]);

        let matches = matcher.find_matches("This is a regular document.");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_no_matches_in_message() {
        let matcher = KeywordMatcher::new(vec!["confidential".to_string()]);

        let matches =
            matcher.search_message(Some("Regular subject"), Some("Regular document body"));

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_count() {
        let mut matches = HashSet::new();
        matches.insert("urgent".to_string());
        matches.insert("action".to_string());

        assert_eq!(KeywordMatcher::match_count(&matches), 2);
    }
}
