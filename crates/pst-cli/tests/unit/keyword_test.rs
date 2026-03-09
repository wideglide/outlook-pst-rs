//! Unit tests for keyword matching (T074)
//!
//! Tests case-insensitivity, multi-keyword search, presence vs count behavior,
//! and missing field handling for `KeywordMatcher`.
#![allow(clippy::similar_names)]

use pst_cli::filter::keyword::KeywordMatcher;
use std::collections::HashSet;

// --- Construction & Parsing ---

#[test]
fn test_from_string_basic() {
    let matcher = KeywordMatcher::from_string("alpha,beta,gamma");
    let kws = matcher.keywords();
    assert_eq!(kws.len(), 3);
    assert!(kws.contains(&"alpha".to_string()));
    assert!(kws.contains(&"beta".to_string()));
    assert!(kws.contains(&"gamma".to_string()));
}

#[test]
fn test_from_string_trims_whitespace() {
    let matcher = KeywordMatcher::from_string("  hello , world  ");
    let kws = matcher.keywords();
    assert_eq!(kws.len(), 2);
    assert!(kws.contains(&"hello".to_string()));
    assert!(kws.contains(&"world".to_string()));
}

#[test]
fn test_from_string_lowercases_all() {
    let matcher = KeywordMatcher::from_string("UPPER,lower,MiXeD");
    let kws = matcher.keywords();
    for kw in &kws {
        assert_eq!(kw, &kw.to_lowercase(), "Keyword should be lowercase: {kw}");
    }
}

#[test]
fn test_from_string_deduplicates_case_insensitive() {
    let matcher = KeywordMatcher::from_string("Test, TEST, test, tEsT");
    assert_eq!(matcher.keywords().len(), 1);
}

#[test]
fn test_from_string_empty_input() {
    let matcher = KeywordMatcher::from_string("");
    assert_eq!(matcher.keywords().len(), 0);
}

#[test]
fn test_from_string_only_commas_and_spaces() {
    let matcher = KeywordMatcher::from_string(" , , , ");
    assert_eq!(matcher.keywords().len(), 0);
}

#[test]
fn test_from_string_single_keyword() {
    let matcher = KeywordMatcher::from_string("urgent");
    assert_eq!(matcher.keywords(), vec!["urgent".to_string()]);
}

// --- Case-Insensitive Matching ---

#[test]
fn test_find_matches_case_insensitive_all_caps() {
    let matcher = KeywordMatcher::from_string("urgent");
    let matches = matcher.find_matches("THIS IS URGENT!");
    assert!(matches.contains("urgent"));
}

#[test]
fn test_find_matches_case_insensitive_mixed_case() {
    let matcher = KeywordMatcher::from_string("contract");
    let matches = matcher.find_matches("The Contract was signed today");
    assert!(matches.contains("contract"));
}

#[test]
fn test_find_matches_case_insensitive_keyword_upper() {
    // Even if keyword is entered as UPPER, from_string lowercases it
    let matcher = KeywordMatcher::from_string("DEAL");
    let matches = matcher.find_matches("closing the deal now");
    assert!(matches.contains("deal"));
}

// --- Multi-Keyword Search ---

#[test]
fn test_multiple_keywords_all_match() {
    let matcher = KeywordMatcher::from_string("merger,acquisition,deal");
    let matches = matcher.find_matches("The merger and acquisition deal closed.");
    assert_eq!(matches.len(), 3);
}

#[test]
fn test_multiple_keywords_some_match() {
    let matcher = KeywordMatcher::from_string("merger,acquisition,deal");
    let matches = matcher.find_matches("The deal closed.");
    assert_eq!(matches.len(), 1);
    assert!(matches.contains("deal"));
}

#[test]
fn test_multiple_keywords_none_match() {
    let matcher = KeywordMatcher::from_string("merger,acquisition,deal");
    let matches = matcher.find_matches("Hello, nice weather today.");
    assert_eq!(matches.len(), 0);
}

// --- Presence vs Count Behavior ---

#[test]
fn test_presence_not_occurrence_count() {
    // "urgent" appears 3 times but should only count as 1 match
    let matcher = KeywordMatcher::from_string("urgent");
    let matches = matcher.find_matches("urgent urgent urgent");
    assert_eq!(matches.len(), 1, "Should count presence, not occurrences");
}

#[test]
fn test_match_count_utility() {
    let mut set = HashSet::new();
    set.insert("a".to_string());
    set.insert("b".to_string());
    set.insert("c".to_string());
    assert_eq!(KeywordMatcher::match_count(&set), 3);
}

#[test]
fn test_match_count_empty() {
    let set = HashSet::new();
    assert_eq!(KeywordMatcher::match_count(&set), 0);
}

// --- search_message with Missing Fields ---

#[test]
fn test_search_message_both_none() {
    let matcher = KeywordMatcher::from_string("urgent");
    let matches = matcher.search_message(None, None);
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_search_message_subject_only_match() {
    let matcher = KeywordMatcher::from_string("urgent,review");
    let matches = matcher.search_message(Some("Urgent review needed"), None);
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_search_message_body_only_match() {
    let matcher = KeywordMatcher::from_string("confidential");
    let matches = matcher.search_message(None, Some("This is confidential material."));
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_search_message_deduplicates_across_subject_and_body() {
    // Keyword appears in both subject and body — should still count as 1
    let matcher = KeywordMatcher::from_string("urgent");
    let matches = matcher.search_message(
        Some("Urgent action required"),
        Some("This is urgent, please respond."),
    );
    assert_eq!(
        matches.len(),
        1,
        "Same keyword in subject+body should count once"
    );
}

#[test]
fn test_search_message_distinct_keywords_in_different_fields() {
    let matcher = KeywordMatcher::from_string("merger,confidential");
    let matches = matcher.search_message(
        Some("The merger is proceeding"),
        Some("This document is confidential"),
    );
    assert_eq!(matches.len(), 2);
    assert!(matches.contains("merger"));
    assert!(matches.contains("confidential"));
}

// --- Substring Matching ---

#[test]
fn test_substring_match() {
    // "act" should match inside "action" and "contract"
    let matcher = KeywordMatcher::from_string("act");
    let matches = matcher.find_matches("Take action on the contract");
    assert_eq!(matches.len(), 1); // "act" found (presence, not count)
}

#[test]
fn test_keyword_with_special_characters() {
    let matcher = KeywordMatcher::from_string("re: action");
    let matches = matcher.find_matches("Subject: Re: Action items");
    assert!(matches.contains("re: action"));
}

// --- Edge Cases ---

#[test]
fn test_empty_text() {
    let matcher = KeywordMatcher::from_string("test");
    let matches = matcher.find_matches("");
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_keyword_longer_than_text() {
    let matcher = KeywordMatcher::from_string("verylongkeyword");
    let matches = matcher.find_matches("short");
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_exact_match() {
    let matcher = KeywordMatcher::from_string("hello");
    let matches = matcher.find_matches("hello");
    assert_eq!(matches.len(), 1);
}
