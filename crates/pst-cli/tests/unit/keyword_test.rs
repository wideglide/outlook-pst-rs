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

// --- US1: HTML-Aware Keyword Matching Regression Tests (T009) ---

use pst_cli::export::html::extract_visible_text;

/// Helper: simulate the HTML-body keyword matching pipeline used in export/mod.rs.
/// Extracts visible text from HTML, then runs keyword matching on the result.
fn search_html_body(matcher: &KeywordMatcher, html: &str) -> HashSet<String> {
    let visible = extract_visible_text(html);
    matcher.find_matches(&visible)
}

#[test]
fn test_html_keyword_matches_visible_text() {
    let matcher = KeywordMatcher::from_string("confidential");
    let html = "<html><body><p>This document is confidential.</p></body></html>";
    let hits = search_html_body(&matcher, html);
    assert_eq!(hits.len(), 1);
    assert!(hits.contains("confidential"));
}

#[test]
fn test_html_keyword_does_not_match_tag_names() {
    let matcher = KeywordMatcher::from_string("body,html,div,span");
    let html = "<html><body><div><span>Hello world</span></div></body></html>";
    let hits = search_html_body(&matcher, html);
    assert_eq!(hits.len(), 0, "Keywords in tag names must not match");
}

#[test]
fn test_html_keyword_does_not_match_attribute_values() {
    let matcher = KeywordMatcher::from_string("secret-class,myid");
    let html = r#"<div class="secret-class" id="myid">Visible text</div>"#;
    let hits = search_html_body(&matcher, html);
    assert_eq!(hits.len(), 0, "Keywords in attribute values must not match");
}

#[test]
fn test_html_keyword_does_not_match_script_content() {
    let matcher = KeywordMatcher::from_string("password");
    let html =
        r#"<html><body><script>var password = "hunter2";</script><p>Safe text</p></body></html>"#;
    let hits = search_html_body(&matcher, html);
    assert_eq!(hits.len(), 0, "Keywords inside <script> must not match");
}

#[test]
fn test_html_keyword_does_not_match_style_content() {
    let matcher = KeywordMatcher::from_string("hidden,display");
    let html = "<html><head><style>.hidden { display: none; }</style></head><body><p>Shown</p></body></html>";
    let hits = search_html_body(&matcher, html);
    assert_eq!(hits.len(), 0, "Keywords inside <style> must not match");
}

#[test]
fn test_html_keyword_does_not_match_html_comments() {
    let matcher = KeywordMatcher::from_string("todo,fixme");
    let html = "<html><body><!-- TODO: fixme later --><p>Normal text</p></body></html>";
    let hits = search_html_body(&matcher, html);
    assert_eq!(
        hits.len(),
        0,
        "Keywords inside HTML comments must not match"
    );
}

#[test]
fn test_html_keyword_matches_only_in_visible_text_mixed() {
    let matcher = KeywordMatcher::from_string("merger,confidential,script");
    let html = r"<html>
        <head><style>.merger { color: red; }</style></head>
        <body>
            <script>var confidential = true;</script>
            <p>The merger is proceeding as planned.</p>
        </body>
    </html>";
    let hits = search_html_body(&matcher, html);
    assert_eq!(
        hits.len(),
        1,
        "Only 'merger' should match from visible text"
    );
    assert!(hits.contains("merger"));
    assert!(
        !hits.contains("confidential"),
        "confidential is only in script"
    );
    assert!(!hits.contains("script"), "script is a tag name");
}

#[test]
fn test_html_keyword_search_message_with_html_body() {
    let matcher = KeywordMatcher::from_string("urgent");
    let visible = extract_visible_text("<html><body><p>This is urgent!</p></body></html>");
    let hits = matcher.search_message(Some("Normal subject"), Some(&visible));
    assert_eq!(hits.len(), 1);
    assert!(hits.contains("urgent"));
}

#[test]
fn test_html_keyword_search_message_subject_match_body_hidden() {
    let matcher = KeywordMatcher::from_string("important,password");
    let visible = extract_visible_text(r#"<script>var password = "x";</script><p>Hello</p>"#);
    let hits = matcher.search_message(Some("This is important"), Some(&visible));
    assert_eq!(hits.len(), 1, "Only subject keyword should match");
    assert!(hits.contains("important"));
    assert!(!hits.contains("password"));
}
