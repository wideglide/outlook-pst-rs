//! Unit tests for message numbering functionality
//!
//! Tests T027: Message numbering and sequence management
//! - Sequence correctness
//! - Zero-padding format
//! - Counter state management

use pst_cli::cli::ExportArgs;
use pst_cli::export::ExportCoordinator;
use std::path::PathBuf;

/// Create a test `ExportCoordinator` with minimal args
fn create_test_coordinator() -> ExportCoordinator {
    let args = ExportArgs {
        input: PathBuf::from("/tmp/test.pst"),
        output: PathBuf::from("/tmp/output"),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    ExportCoordinator::new(args)
}

#[test]
fn test_sequence_starts_at_one() {
    let mut coordinator = create_test_coordinator();
    let first = coordinator.next_sequence_number();

    assert_eq!(first, 1, "First sequence number should be 1");
}

#[test]
fn test_sequence_increments_correctly() {
    let mut coordinator = create_test_coordinator();

    let seq1 = coordinator.next_sequence_number();
    let seq2 = coordinator.next_sequence_number();
    let seq3 = coordinator.next_sequence_number();

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_eq!(seq3, 3);
}

#[test]
fn test_sequence_continuous_incrementing() {
    let mut coordinator = create_test_coordinator();

    for expected in 1..=100 {
        let seq = coordinator.next_sequence_number();
        assert_eq!(
            seq, expected,
            "Sequence should increment continuously without gaps"
        );
    }
}

#[test]
fn test_format_sequence_zero_padding() {
    // Test various sequence numbers for correct zero-padding
    assert_eq!(ExportCoordinator::format_sequence(1), "00001");
    assert_eq!(ExportCoordinator::format_sequence(2), "00002");
    assert_eq!(ExportCoordinator::format_sequence(10), "00010");
    assert_eq!(ExportCoordinator::format_sequence(99), "00099");
    assert_eq!(ExportCoordinator::format_sequence(100), "00100");
    assert_eq!(ExportCoordinator::format_sequence(999), "00999");
    assert_eq!(ExportCoordinator::format_sequence(1000), "01000");
    assert_eq!(ExportCoordinator::format_sequence(9999), "09999");
    assert_eq!(ExportCoordinator::format_sequence(10000), "10000");
}

#[test]
fn test_format_sequence_always_five_digits() {
    for num in [1, 5, 10, 50, 100, 500, 1000, 5000, 10000, 50000, 99999] {
        let formatted = ExportCoordinator::format_sequence(num);
        assert_eq!(
            formatted.len(),
            5,
            "Formatted number {num} should always be 5 characters long"
        );
    }
}

#[test]
fn test_format_sequence_leading_zeros() {
    let formatted = ExportCoordinator::format_sequence(42);
    assert_eq!(formatted, "00042");
    assert!(formatted.starts_with("000"));
}

#[test]
fn test_format_sequence_large_numbers() {
    // Test numbers larger than 99999 (should still work but be > 5 digits)
    let formatted = ExportCoordinator::format_sequence(100_000);
    assert_eq!(formatted.len(), 6);
    assert_eq!(formatted, "100000");
}

#[test]
fn test_sequence_state_persistence() {
    let mut coordinator = create_test_coordinator();

    // Get first 5 sequence numbers
    coordinator.next_sequence_number(); // 1
    coordinator.next_sequence_number(); // 2
    coordinator.next_sequence_number(); // 3
    coordinator.next_sequence_number(); // 4
    coordinator.next_sequence_number(); // 5

    // Next should be 6
    let next = coordinator.next_sequence_number();
    assert_eq!(next, 6, "Counter state should persist across calls");
}

#[test]
fn test_get_message_output_dir_main() {
    let coordinator = create_test_coordinator();

    let dir = coordinator.get_message_output_dir(1, false);
    let path_str = dir.to_str().unwrap();

    // Should be output/00001 (not in duplicates/)
    assert!(path_str.ends_with("00001"));
    assert!(!path_str.contains("duplicates"));
}

#[test]
fn test_get_message_output_dir_duplicate() {
    let coordinator = create_test_coordinator();

    let dir = coordinator.get_message_output_dir(10, true);
    let path_str = dir.to_str().unwrap();

    // Should be output/duplicates/00010
    assert!(path_str.contains("duplicates"));
    assert!(path_str.ends_with("00010"));
}

#[test]
fn test_get_message_output_dir_sequence_formatting() {
    let coordinator = create_test_coordinator();

    let dir1 = coordinator.get_message_output_dir(1, false);
    let dir25 = coordinator.get_message_output_dir(25, false);
    let dir999 = coordinator.get_message_output_dir(999, false);

    assert!(dir1.to_str().unwrap().ends_with("00001"));
    assert!(dir25.to_str().unwrap().ends_with("00025"));
    assert!(dir999.to_str().unwrap().ends_with("00999"));
}

#[test]
fn test_sequence_deterministic_across_multiple_coordinators() {
    // Each coordinator has independent counter starting at 1
    let mut coord1 = create_test_coordinator();
    let mut coord2 = create_test_coordinator();

    let seq1_a = coord1.next_sequence_number();
    let seq2_a = coord2.next_sequence_number();

    assert_eq!(seq1_a, 1);
    assert_eq!(seq2_a, 1);

    let seq1_b = coord1.next_sequence_number();
    let seq2_b = coord2.next_sequence_number();

    assert_eq!(seq1_b, 2);
    assert_eq!(seq2_b, 2);
}

#[test]
fn test_format_sequence_consistency() {
    // Same number should always produce same formatted result
    for _ in 0..100 {
        let formatted = ExportCoordinator::format_sequence(42);
        assert_eq!(formatted, "00042");
    }
}

#[test]
fn test_sequence_boundary_values() {
    let mut coordinator = create_test_coordinator();

    // Test at boundary values
    for _ in 1..=99999 {
        let seq = coordinator.next_sequence_number();
        assert!(seq > 0, "Sequence should always be positive");

        // Format should succeed
        let formatted = ExportCoordinator::format_sequence(seq);
        assert!(formatted.len() >= 5, "Format should be at least 5 digits");
    }
}

#[test]
fn test_format_sequence_edge_cases() {
    // Test edge cases for formatting

    // Zero (edge case - not typically used but should handle)
    let formatted_zero = ExportCoordinator::format_sequence(0);
    assert_eq!(formatted_zero, "00000");

    // Maximum typical range
    let formatted_max = ExportCoordinator::format_sequence(99999);
    assert_eq!(formatted_max, "99999");
}

#[test]
fn test_output_dir_path_construction() {
    let args = ExportArgs {
        input: PathBuf::from("/tmp/test.pst"),
        output: PathBuf::from("/export/output"),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let coordinator = ExportCoordinator::new(args);

    let dir = coordinator.get_message_output_dir(123, false);
    let expected = PathBuf::from("/export/output").join("00123");

    assert_eq!(dir, expected);
}

#[test]
fn test_duplicate_dir_path_construction() {
    let args = ExportArgs {
        input: PathBuf::from("/tmp/test.pst"),
        output: PathBuf::from("/export/output"),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let coordinator = ExportCoordinator::new(args);

    let dir = coordinator.get_message_output_dir(456, true);
    let expected = PathBuf::from("/export/output")
        .join("duplicates")
        .join("00456");

    assert_eq!(dir, expected);
}
