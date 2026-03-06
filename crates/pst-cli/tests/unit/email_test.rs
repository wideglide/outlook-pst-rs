//! Unit tests for email participant matching (US7)
//!
//! Tests address extraction from display name format, case-insensitivity,
//! multi-field search, and de-duplication.

use pst_cli::filter::email::{EmailMatcher, extract_email_address};

// --- Address extraction from "Display Name <address>" format ---

#[test]
fn test_extract_plain_email() {
    assert_eq!(extract_email_address("user@example.com"), "user@example.com");
}

#[test]
fn test_extract_display_name_format() {
    assert_eq!(
        extract_email_address("John Doe <john@example.com>"),
        "john@example.com"
    );
}

#[test]
fn test_extract_strips_whitespace_in_brackets() {
    assert_eq!(
        extract_email_address("  Name  <  user@host.com  >  "),
        "user@host.com"
    );
}

#[test]
fn test_extract_only_angle_brackets() {
    assert_eq!(
        extract_email_address("<addr@domain.org>"),
        "addr@domain.org"
    );
}

#[test]
fn test_extract_complex_display_name() {
    assert_eq!(
        extract_email_address("\"Doe, John\" <john.doe@company.com>"),
        "john.doe@company.com"
    );
}

#[test]
fn test_extract_empty() {
    assert_eq!(extract_email_address(""), "");
}

// --- Case-insensitivity ---

#[test]
fn test_case_insensitive_matching() {
    let matcher = EmailMatcher::new(vec!["john@example.com".to_string()]);
    let matches = matcher.find_matches_str(&["JOHN@EXAMPLE.COM".to_string()]);
    assert_eq!(matches.len(), 1);
    assert!(matches.contains("john@example.com"));
}

#[test]
fn test_case_insensitive_matching_mixed_case() {
    let matcher = EmailMatcher::new(vec!["John.Doe@Example.COM".to_string()]);
    let matches = matcher.find_matches_str(&["john.doe@example.com".to_string()]);
    assert_eq!(matches.len(), 1);
}

// --- Multi-keyword / multi-field search ---

#[test]
fn test_multi_target_matching() {
    let matcher = EmailMatcher::new(vec![
        "alice@example.com".to_string(),
        "bob@example.com".to_string(),
        "charlie@test.org".to_string(),
    ]);

    let matches = matcher.search_message(
        "alice@example.com",
        &["bob@example.com".to_string()],
        &["charlie@test.org".to_string()],
        &[],
    );
    assert_eq!(matches.len(), 3);
}

#[test]
fn test_search_across_all_fields() {
    let matcher = EmailMatcher::new(vec![
        "from@example.com".to_string(),
        "to@example.com".to_string(),
        "cc@example.com".to_string(),
        "bcc@example.com".to_string(),
    ]);

    let matches = matcher.search_message(
        "from@example.com",
        &["to@example.com".to_string()],
        &["cc@example.com".to_string()],
        &["bcc@example.com".to_string()],
    );
    assert_eq!(matches.len(), 4);
}

#[test]
fn test_search_from_only() {
    let matcher = EmailMatcher::new(vec!["target@example.com".to_string()]);

    let matches = matcher.search_message(
        "Target <target@example.com>",
        &[],
        &[],
        &[],
    );
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_search_bcc_only() {
    let matcher = EmailMatcher::new(vec!["hidden@example.com".to_string()]);

    let matches = matcher.search_message(
        "sender@other.com",
        &[],
        &[],
        &["Hidden <hidden@example.com>".to_string()],
    );
    assert_eq!(matches.len(), 1);
}

// --- De-duplication ---

#[test]
fn test_deduplication_same_email_different_fields() {
    let matcher = EmailMatcher::new(vec!["person@example.com".to_string()]);

    let matches = matcher.search_message(
        "person@example.com",
        &["person@example.com".to_string()],
        &["person@example.com".to_string()],
        &["person@example.com".to_string()],
    );
    // Should appear only once despite being in all fields
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_deduplication_in_target_list() {
    let matcher = EmailMatcher::new(vec![
        "same@example.com".to_string(),
        "SAME@EXAMPLE.COM".to_string(),
    ]);
    // After dedup, only one target
    assert_eq!(matcher.emails().len(), 1);
}

// --- No false positives ---

#[test]
fn test_no_match_when_no_targets() {
    let matcher = EmailMatcher::from_string("");
    let matches = matcher.search_message(
        "sender@example.com",
        &["to@example.com".to_string()],
        &[],
        &[],
    );
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_no_match_with_different_domain() {
    let matcher = EmailMatcher::new(vec!["user@example.com".to_string()]);
    let matches = matcher.find_matches_str(&["user@different.com".to_string()]);
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_no_false_positive_from_partial_address() {
    // "user@example" should not match "user@example.com"
    let matcher = EmailMatcher::new(vec!["user@example".to_string()]);
    let matches = matcher.find_matches_str(&["user@example.com".to_string()]);
    assert_eq!(matches.len(), 0);
}

// --- from_string parsing ---

#[test]
fn test_from_string_basic() {
    let matcher = EmailMatcher::from_string("a@test.com, b@test.com");
    assert_eq!(matcher.emails().len(), 2);
}

#[test]
fn test_from_string_with_display_names() {
    let matcher = EmailMatcher::from_string("Alice <a@test.com>, Bob <b@test.com>");
    let emails = matcher.emails();
    assert_eq!(emails.len(), 2);
    assert!(emails.contains(&"a@test.com".to_string()));
    assert!(emails.contains(&"b@test.com".to_string()));
}

#[test]
fn test_from_string_handles_empties() {
    let matcher = EmailMatcher::from_string(",  , valid@example.com,  ,");
    assert_eq!(matcher.emails().len(), 1);
}

// --- Missing field handling ---

#[test]
fn test_empty_to_cc_bcc() {
    let matcher = EmailMatcher::new(vec!["any@example.com".to_string()]);
    let matches = matcher.search_message("sender@other.com", &[], &[], &[]);
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_from_field_with_display_name() {
    let matcher = EmailMatcher::new(vec!["john@example.com".to_string()]);
    let matches = matcher.search_message(
        "John Doe <john@example.com>",
        &[],
        &[],
        &[],
    );
    assert_eq!(matches.len(), 1);
}

// --- match_count helper ---

#[test]
fn test_match_count_empty() {
    let matches = std::collections::HashSet::new();
    assert_eq!(EmailMatcher::match_count(&matches), 0);
}

#[test]
fn test_match_count_multiple() {
    let mut matches = std::collections::HashSet::new();
    matches.insert("a@example.com".to_string());
    matches.insert("b@example.com".to_string());
    matches.insert("c@example.com".to_string());
    assert_eq!(EmailMatcher::match_count(&matches), 3);
}
