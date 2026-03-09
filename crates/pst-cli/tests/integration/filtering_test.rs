//! Integration tests for keyword and email filtering (US6 + US7)
//!
//! Tests T075: Keyword filtering with known test corpus
//! Tests T085: Email participant filtering with known test corpus

use pst_cli::cli::progress::ProgressReporter;
use pst_cli::cli::ExportArgs;
use pst_cli::export::ExportCoordinator;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to get a test PST file from fixtures
fn get_test_pst_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("outlook.pst")
}

/// Helper to create export args
fn create_export_args(input: PathBuf, output: PathBuf) -> ExportArgs {
    ExportArgs {
        input,
        output,
        metadata: true, // Enable metadata so we can check keyword/email reporting
        attachments: false,
        headers: false,
        csv: true, // Enable CSV so we can check counts
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    }
}

// --- Keyword Filtering Integration Tests (T075) ---

#[test]
fn test_export_with_keyword_filtering() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    args.keywords = Some(vec!["test".to_string(), "hello".to_string()]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(
        result.is_ok(),
        "Export with keywords should succeed: {:?}",
        result.err()
    );

    // Verify metadata files contain keyword sections
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should read output dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    for entry in &entries {
        let metadata_path = entry.path().join("metadata.txt");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).expect("Should read metadata.txt");
            // Every metadata file should have a Keywords line
            assert!(
                content.contains("Keywords:"),
                "metadata.txt should contain Keywords section"
            );
        }
    }

    // Verify CSV file contains keyword count column
    let csv_path = output_path.join("emails.csv");
    if csv_path.exists() {
        let csv_content = fs::read_to_string(&csv_path).expect("Should read CSV");
        assert!(
            csv_content.contains("KeywordCount"),
            "CSV should contain KeywordCount column header"
        );
    }
}

#[test]
fn test_export_keywords_metadata_reports_none_when_no_match() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    // Use a keyword very unlikely to match
    args.keywords = Some(vec!["xyzzynonexistentkeyword123".to_string()]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok(), "Export should succeed: {:?}", result.err());

    // Check metadata files show "Keywords: none" since nothing matched
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should read output dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    for entry in &entries {
        let metadata_path = entry.path().join("metadata.txt");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).expect("Should read metadata.txt");
            assert!(
                content.contains("Keywords: none"),
                "Keywords should be 'none' when keyword doesn't match, got: {content}"
            );
        }
    }
}

// --- Email Filtering Integration Tests (T085) ---

#[test]
fn test_export_with_email_filtering() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    // Search for an email that might be in the test PST
    args.emails = Some(vec![
        "test@example.com".to_string(),
        "user@example.com".to_string(),
    ]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(
        result.is_ok(),
        "Export with emails should succeed: {:?}",
        result.err()
    );

    // Verify metadata files contain email match sections
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should read output dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    for entry in &entries {
        let metadata_path = entry.path().join("metadata.txt");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).expect("Should read metadata.txt");
            // Every metadata file should have an Email Matches line
            assert!(
                content.contains("Email Matches:"),
                "metadata.txt should contain Email Matches section"
            );
        }
    }

    // Verify CSV file contains EmailMatchCount column
    let csv_path = output_path.join("emails.csv");
    if csv_path.exists() {
        let csv_content = fs::read_to_string(&csv_path).expect("Should read CSV");
        assert!(
            csv_content.contains("EmailMatchCount"),
            "CSV should contain EmailMatchCount column header"
        );
    }
}

#[test]
fn test_export_emails_metadata_reports_none_when_no_match() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    // Use an email very unlikely to match
    args.emails = Some(vec!["nonexistent99999@nowhere.invalid".to_string()]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok(), "Export should succeed: {:?}", result.err());

    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should read output dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    for entry in &entries {
        let metadata_path = entry.path().join("metadata.txt");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).expect("Should read metadata.txt");
            assert!(
                content.contains("Email Matches: none"),
                "Email Matches should be 'none' when no email matches, got: {content}"
            );
        }
    }
}

#[test]
fn test_export_combined_keyword_and_email_filtering() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    args.keywords = Some(vec!["test".to_string()]);
    args.emails = Some(vec!["user@example.com".to_string()]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(
        result.is_ok(),
        "Export with both filters should succeed: {:?}",
        result.err()
    );

    // Verify CSV has both count columns
    let csv_path = output_path.join("emails.csv");
    if csv_path.exists() {
        let csv_content = fs::read_to_string(&csv_path).expect("Should read CSV");
        assert!(
            csv_content.contains("KeywordCount"),
            "CSV should have KeywordCount"
        );
        assert!(
            csv_content.contains("EmailMatchCount"),
            "CSV should have EmailMatchCount"
        );
    }

    // Verify metadata has both sections
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should read output dir")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    for entry in &entries {
        let metadata_path = entry.path().join("metadata.txt");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).expect("Should read metadata.txt");
            assert!(
                content.contains("Keywords:"),
                "Should have Keywords section"
            );
            assert!(
                content.contains("Email Matches:"),
                "Should have Email Matches section"
            );
        }
    }
}

#[test]
fn test_csv_email_match_count_accuracy() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let mut args = create_export_args(pst_path, output_path.clone());
    // Use a clearly non-matching email to ensure all rows have count 0
    args.emails = Some(vec!["veryrandomnomatch@nowhere999.invalid".to_string()]);

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok(), "Export should succeed");

    let csv_path = output_path.join("emails.csv");
    if csv_path.exists() {
        let csv_content = fs::read_to_string(&csv_path).expect("Should read CSV");
        let lines: Vec<&str> = csv_content.lines().collect();
        let header_fields: Vec<&str> = lines
            .first()
            .expect("CSV should include a header row")
            .split(',')
            .collect();
        let email_match_count_idx = header_fields
            .iter()
            .position(|field| *field == "EmailMatchCount")
            .expect("CSV header should include EmailMatchCount column");

        // Skip header, check each data row
        for line in lines.iter().skip(1) {
            // Pull EmailMatchCount by column name so schema extension is safe.
            let fields: Vec<&str> = line.split(',').collect();
            if let Some(count_field) = fields.get(email_match_count_idx) {
                let count: usize = count_field.trim().parse().unwrap_or(999);
                assert_eq!(
                    count, 0,
                    "All email match counts should be 0 for non-matching address"
                );
            }
        }
    }
}
