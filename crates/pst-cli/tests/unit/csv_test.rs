//! Unit tests for CSV export functionality

use pst_cli::export::csv::{CsvExporter, CsvRow};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_csv_header_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    assert_eq!(
        content.trim(),
        "SequenceNumber,Subject,From,To,Date,MessageId,IsDuplicate,KeywordCount,EmailMatchCount,Size,AttachmentCount,ConvNumber,PST-StoreName,Error"
    );
}

#[test]
fn test_csv_row_simple_data() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "Test Subject".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11T10:00:00Z".to_string(),
        message_id: "msg123@example.com".to_string(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 1024,
        attachment_count: 2,
        conv_number: "00001".to_string(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[1],
        "1,Test Subject,sender@example.com,recipient@example.com,2026-02-11T10:00:00Z,msg123@example.com,false,0,0,1024,2,00001,mailbox.pst,0"
    );
}

#[test]
fn test_csv_row_with_comma_in_subject() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "Subject, with comma".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "msg123".to_string(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    // Subject should be quoted because it contains a comma
    assert!(lines[1].contains("\"Subject, with comma\""));
}

#[test]
fn test_csv_row_with_quotes_in_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "He said \"hello\"".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "msg123".to_string(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    // Quotes should be doubled and field should be quoted
    assert!(lines[1].contains("\"He said \"\"hello\"\"\""));
}

#[test]
fn test_csv_row_with_newline_in_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "Line 1\nLine 2".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "msg123".to_string(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");

    // Field with newline should be quoted
    assert!(content.contains("\"Line 1\nLine 2\""));
}

#[test]
fn test_csv_multiple_rows() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    for i in 1..=5 {
        let row = CsvRow {
            sequence_number: i,
            subject: format!("Subject {i}"),
            from: format!("sender{i}@example.com"),
            to: format!("recipient{i}@example.com"),
            date: "2026-02-11".to_string(),
            message_id: format!("msg{i}"),
            is_duplicate: i > 3, // Last two are duplicates
            keyword_count: 0,
            email_match_count: 0,
            size: 0,
            attachment_count: 0,
            conv_number: String::new(),
            pst_store_name: "mailbox.pst".to_string(),
            error: 0,
        };
        exporter.write_row(&row).expect("Failed to write row");
    }

    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    // Header + 5 rows
    assert_eq!(lines.len(), 6);

    // Check duplicate flags
    assert!(lines[4].contains(",true,")); // Row 4 is duplicate
    assert!(lines[5].contains(",true,")); // Row 5 is duplicate
    assert!(lines[1].contains(",false,")); // Row 1 is not duplicate
}

#[test]
fn test_csv_rows_written_count() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    assert_eq!(exporter.rows_written(), 0);

    for i in 1..=3 {
        let row = CsvRow {
            sequence_number: i,
            subject: format!("Subject {i}"),
            from: "sender@example.com".to_string(),
            to: "recipient@example.com".to_string(),
            date: "2026-02-11".to_string(),
            message_id: format!("msg{i}"),
            is_duplicate: false,
            keyword_count: 0,
            email_match_count: 0,
            size: 0,
            attachment_count: 0,
            conv_number: String::new(),
            pst_store_name: "mailbox.pst".to_string(),
            error: 0,
        };
        exporter.write_row(&row).expect("Failed to write row");
    }

    assert_eq!(exporter.rows_written(), 3);
}

#[test]
fn test_csv_duplicate_status_column() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    // Write a non-duplicate
    let row1 = CsvRow {
        sequence_number: 1,
        subject: "Unique message".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "unique@example.com".to_string(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };
    exporter.write_row(&row1).expect("Failed to write row");

    // Write a duplicate
    let row2 = CsvRow {
        sequence_number: 2,
        subject: "Duplicate message".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "unique@example.com".to_string(),
        is_duplicate: true,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 1,
    };
    exporter.write_row(&row2).expect("Failed to write row");

    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 3);
    assert!(
        lines[1].contains(",false,"),
        "First row should be false for duplicate"
    );
    assert!(
        lines[2].contains(",true,"),
        "Second row should be true for duplicate"
    );
}

#[test]
fn test_csv_empty_message_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: String::new(),
        is_duplicate: false,
        keyword_count: 0,
        email_match_count: 0,
        size: 0,
        attachment_count: 0,
        conv_number: String::new(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    // Should handle empty message_id gracefully
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_csv_keyword_and_email_counts() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let csv_path = temp_dir.path().join("test.csv");

    let mut exporter = CsvExporter::new(&csv_path).expect("Failed to create CSV exporter");
    exporter.write_header().expect("Failed to write header");

    let row = CsvRow {
        sequence_number: 1,
        subject: "Test".to_string(),
        from: "sender@example.com".to_string(),
        to: "recipient@example.com".to_string(),
        date: "2026-02-11".to_string(),
        message_id: "msg123".to_string(),
        is_duplicate: false,
        keyword_count: 3,
        email_match_count: 2,
        size: 4096,
        attachment_count: 1,
        conv_number: "00012".to_string(),
        pst_store_name: "mailbox.pst".to_string(),
        error: 0,
    };

    exporter.write_row(&row).expect("Failed to write row");
    exporter.flush().expect("Failed to flush");

    let content = fs::read_to_string(&csv_path).expect("Failed to read CSV");
    let lines: Vec<&str> = content.lines().collect();

    // Should include keyword/email counts and the appended new columns.
    assert!(lines[1].contains(",3,2,4096,1,00012,mailbox.pst,0"));
}
