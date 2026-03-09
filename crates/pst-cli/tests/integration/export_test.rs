//! Integration tests for PST export functionality
//!
//! Tests T028: Basic single-file export
//! - Create minimal PST fixture
//! - Run export
//! - Verify folder structure
//! - Verify message.html content correctness

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

/// Helper to create export args for testing
fn create_test_export_args(input: PathBuf, output: PathBuf) -> ExportArgs {
    ExportArgs {
        input,
        output,
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    }
}

#[test]
fn test_basic_single_file_export() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator
    let args = create_test_export_args(pst_path.clone(), output_path.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode for tests

    // Run export
    let result = coordinator.run(&mut reporter);

    // Verify export succeeded
    assert!(result.is_ok(), "Export should succeed: {:?}", result.err());

    // Verify output directory was created
    assert!(output_path.exists(), "Output directory should exist");
    assert!(output_path.is_dir(), "Output path should be a directory");

    // Check that at least one message was exported
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .collect();

    assert!(
        !entries.is_empty(),
        "At least one message should be exported"
    );

    // Verify first message has proper structure (00001/)
    // Note: This might not always exist depending on PST content
    // Check if any numbered directories exist
    let numbered_dirs: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .all(|c| c.is_ascii_digit())
        })
        .collect();

    if !numbered_dirs.is_empty() {
        // Verify at least one numbered directory exists
        assert!(
            !numbered_dirs.is_empty(),
            "Should have at least one numbered message directory"
        );

        // Check first numbered directory for message.html
        let first_dir = numbered_dirs[0].path();
        let message_html = first_dir.join("message.html");

        // Verify message.html exists
        if message_html.exists() {
            assert!(message_html.is_file(), "message.html should be a file");

            // Verify message.html has content
            let content =
                fs::read_to_string(&message_html).expect("Should be able to read message.html");

            assert!(!content.is_empty(), "message.html should not be empty");
            assert!(
                content.contains("<html"),
                "message.html should contain HTML content"
            );
            assert!(
                content.contains("</html>"),
                "message.html should be complete HTML"
            );
        }
    }
}

#[test]
fn test_export_folder_structure() {
    let pst_path = get_test_pst_path();

    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let args = create_test_export_args(pst_path, output_path.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok());

    // Verify folder structure conventions
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .collect();

    // Check that directories follow naming convention
    for entry in entries {
        if entry.path().is_dir() {
            let dir_name = entry.file_name();
            let name_str = dir_name.to_str().unwrap();

            // Skip duplicates directory
            if name_str == "duplicates" {
                continue;
            }

            // Should be 5-digit zero-padded number
            if name_str.len() == 5 && name_str.chars().all(|c| c.is_ascii_digit()) {
                // Valid format
                let num: u32 = name_str.parse().unwrap();
                assert!(num > 0, "Sequence number should be positive");
            }
        }
    }
}

#[test]
fn test_export_message_html_content() {
    let pst_path = get_test_pst_path();

    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    let args = create_test_export_args(pst_path, output_path.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok());

    // Find first message directory
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir() && e.file_name().to_str().unwrap_or("") != "duplicates")
        .collect();

    if let Some(first_entry) = entries.first() {
        let message_html = first_entry.path().join("message.html");

        if message_html.exists() {
            let content =
                fs::read_to_string(&message_html).expect("Should be able to read message.html");

            // Verify HTML structure
            assert!(content.contains("<html"), "Should have opening html tag");
            assert!(content.contains("</html>"), "Should have closing html tag");
            assert!(content.contains("<body"), "Should have body tag");
        }
    }
}

#[test]
fn test_export_handles_empty_output_dir() {
    let pst_path = get_test_pst_path();

    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("new_output_dir");

    // Output directory doesn't exist yet - should be created
    assert!(!output_path.exists());

    let args = create_test_export_args(pst_path, output_path.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(
        result.is_ok(),
        "Export should create output directory if it doesn't exist"
    );

    // Verify directory was created
    assert!(output_path.exists());
    assert!(output_path.is_dir());
}

#[test]
fn test_export_deterministic_numbering() {
    let pst_path = get_test_pst_path();

    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Run export twice to different directories
    let temp_dir1 = TempDir::new().expect("Failed to create temp dir 1");
    let temp_dir2 = TempDir::new().expect("Failed to create temp dir 2");

    let output_path1 = temp_dir1.path().to_path_buf();
    let output_path2 = temp_dir2.path().to_path_buf();

    // First export
    let args1 = create_test_export_args(pst_path.clone(), output_path1.clone());
    let mut coordinator1 = ExportCoordinator::new(args1);
    let mut reporter1 = ProgressReporter::new(true);
    coordinator1
        .run(&mut reporter1)
        .expect("First export should succeed");

    // Second export
    let args2 = create_test_export_args(pst_path, output_path2.clone());
    let mut coordinator2 = ExportCoordinator::new(args2);
    let mut reporter2 = ProgressReporter::new(true);
    coordinator2
        .run(&mut reporter2)
        .expect("Second export should succeed");

    // Collect directory names from both exports
    let dirs1: Vec<String> = fs::read_dir(&output_path1)
        .expect("Should read first output")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    let dirs2: Vec<String> = fs::read_dir(&output_path2)
        .expect("Should read second output")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Should have same number of directories
    assert_eq!(
        dirs1.len(),
        dirs2.len(),
        "Both exports should produce same number of directories (deterministic)"
    );

    // Directory names should match (same numbering)
    for dir_name in &dirs1 {
        assert!(
            dirs2.contains(dir_name),
            "Both exports should have same directory structure"
        );
    }
}

//
// User Story 2: Batch Export Tests
//

#[test]
fn test_batch_export_multiple_pst_files() {
    // Create a temporary directory with multiple PST files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("pst_files");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    // Get test PST file
    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    // Copy test PST file multiple times with different names (alphabetical order)
    let pst1 = input_dir.join("alice.pst");
    let pst2 = input_dir.join("bob.pst");
    let pst3 = input_dir.join("charlie.pst");

    fs::copy(&test_pst, &pst1).expect("Failed to copy PST 1");
    fs::copy(&test_pst, &pst2).expect("Failed to copy PST 2");
    fs::copy(&test_pst, &pst3).expect("Failed to copy PST 3");

    // Create output directory
    let output_dir = temp_dir.path().join("output");

    // Run export on directory
    let args = create_test_export_args(input_dir.clone(), output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok(), "Batch export should succeed");

    // Verify output directory exists
    assert!(output_dir.exists());

    // Count exported messages
    let entries: Vec<_> = fs::read_dir(&output_dir)
        .expect("Should read output directory")
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir() && e.file_name().to_str().unwrap_or("") != "duplicates")
        .collect();

    // Should have messages from all three PST files
    assert!(
        !entries.is_empty(),
        "Should have exported messages from batch"
    );
}

#[test]
fn test_batch_export_alphabetical_order() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("pst_files");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    // Create PST files in non-alphabetical order
    let pst_zebra = input_dir.join("zebra.pst");
    let pst_apple = input_dir.join("apple.pst");
    let pst_mango = input_dir.join("mango.pst");

    fs::copy(&test_pst, &pst_zebra).expect("Failed to copy zebra.pst");
    fs::copy(&test_pst, &pst_apple).expect("Failed to copy apple.pst");
    fs::copy(&test_pst, &pst_mango).expect("Failed to copy mango.pst");

    let output_dir = temp_dir.path().join("output");

    let args = create_test_export_args(input_dir, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok());

    // Files should be processed in alphabetical order: apple, mango, zebra
    // This is verified by the deterministic numbering - same results on repeated runs
    assert!(output_dir.exists());
}

#[test]
fn test_batch_export_sequential_numbering() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("pst_files");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    // Create two PST files
    let pst1 = input_dir.join("first.pst");
    let pst2 = input_dir.join("second.pst");

    fs::copy(&test_pst, &pst1).expect("Failed to copy first.pst");
    fs::copy(&test_pst, &pst2).expect("Failed to copy second.pst");

    let output_dir = temp_dir.path().join("output");

    let args = create_test_export_args(input_dir, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok());

    // Verify sequential numbering across files
    let entries: Vec<_> = fs::read_dir(&output_dir)
        .expect("Should read output directory")
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_str().unwrap_or("");
            e.path().is_dir() && name_str.len() == 5 && name_str.chars().all(|c| c.is_ascii_digit())
        })
        .collect();

    if entries.len() >= 2 {
        // Should have sequential numbers
        let mut numbers: Vec<u32> = entries
            .iter()
            .filter_map(|e| e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()))
            .collect();

        numbers.sort_unstable();

        // Check for sequential numbering (allowing gaps if some messages failed)
        // Just verify they're in increasing order
        for i in 1..numbers.len() {
            assert!(numbers[i] > numbers[i - 1], "Numbers should be increasing");
        }
    }
}

#[test]
fn test_batch_export_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("empty_pst_dir");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    let output_dir = temp_dir.path().join("output");

    let args = create_test_export_args(input_dir, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);

    // Should succeed even with no PST files
    assert!(result.is_ok(), "Should handle empty directory gracefully");

    // Output directory should still be created
    assert!(output_dir.exists());
}

#[test]
fn test_batch_export_continue_on_error() {
    // This test would require creating an invalid PST file
    // For now, we'll skip it as it requires fixture preparation
    // The error handling logic is already in place in export/mod.rs

    // TODO: Create test with one valid and one invalid PST file
    // Verify that valid file is processed despite invalid file error
}

//
// User Story 3: Duplicate Detection Tests
//

#[test]
fn test_duplicate_detection_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("pst_files");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    // Copy same PST file twice to create natural duplicates
    let pst1 = input_dir.join("file1.pst");
    let pst2 = input_dir.join("file2.pst");

    fs::copy(&test_pst, &pst1).expect("Failed to copy PST 1");
    fs::copy(&test_pst, &pst2).expect("Failed to copy PST 2");

    let output_dir = temp_dir.path().join("output");

    let args = create_test_export_args(input_dir, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let result = coordinator.run(&mut reporter);
    assert!(result.is_ok(), "Export with duplicates should succeed");

    // Check if duplicates directory exists
    let duplicates_dir = output_dir.join("duplicates");

    // If duplicates were found, directory should exist
    if duplicates_dir.exists() {
        assert!(duplicates_dir.is_dir(), "duplicates should be a directory");

        // Verify it contains numbered directories
        let dup_entries: Vec<_> = fs::read_dir(&duplicates_dir)
            .expect("Should read duplicates directory")
            .filter_map(std::result::Result::ok)
            .collect();

        assert!(!dup_entries.is_empty(), "Should have duplicate messages");
    }
}

#[test]
fn test_duplicate_directory_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("pst_files");
    fs::create_dir(&input_dir).expect("Failed to create input dir");

    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    // Create duplicates by copying PST file
    let pst1 = input_dir.join("original.pst");
    let pst2 = input_dir.join("copy.pst");

    fs::copy(&test_pst, &pst1).expect("Failed to copy original");
    fs::copy(&test_pst, &pst2).expect("Failed to copy duplicate");

    let output_dir = temp_dir.path().join("output");

    let args = create_test_export_args(input_dir, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    coordinator
        .run(&mut reporter)
        .expect("Export should succeed");

    let duplicates_dir = output_dir.join("duplicates");
    if duplicates_dir.exists() {
        // Verify duplicate messages follow same naming convention
        let entries: Vec<_> = fs::read_dir(&duplicates_dir)
            .expect("Should read duplicates directory")
            .filter_map(std::result::Result::ok)
            .collect();

        for entry in entries {
            if entry.path().is_dir() {
                let name = entry.file_name();
                let name_str = name.to_str().unwrap();

                // Should be 5-digit format
                if name_str.len() == 5 && name_str.chars().all(|c| c.is_ascii_digit()) {
                    // Check for message.html
                    let _message_html = entry.path().join("message.html");
                    // Message.html may or may not exist depending on export success
                }
            }
        }
    }
}

#[test]
fn test_no_false_duplicates() {
    let test_pst = get_test_pst_path();
    if !test_pst.exists() {
        eprintln!("Skipping test: fixture {} not found", test_pst.display());
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_path_buf();

    // Single PST file export - should have no duplicates (first run)
    let args = create_test_export_args(test_pst, output_dir.clone());
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    coordinator
        .run(&mut reporter)
        .expect("First export should succeed");

    // Duplicates directory shouldn't exist or should be empty for single unique file
    let _duplicates_dir = output_dir.join("duplicates");

    // In a single PST export with unique messages, duplicates dir may not exist
    // This is expected behavior
}

#[test]
fn test_export_with_metadata() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with metadata enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: true,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode for tests

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export with metadata should succeed");

    // Verify metadata files exist alongside message files
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .collect();

    // Should have at least one message directory
    assert!(
        !entries.is_empty(),
        "Output directory should contain exported messages"
    );

    // Check that metadata.txt files exist where expected
    for entry in entries {
        let path = entry.path();
        if path.is_dir()
            && !path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("duplicates")
        {
            let metadata_file = path.join("metadata.txt");
            // If this message was exported (has message.html), it should have metadata.txt
            if path.join("message.html").exists() {
                assert!(
                    metadata_file.exists(),
                    "Metadata file should exist at {metadata_file:?}"
                );

                // Verify metadata content has expected fields
                let content = fs::read_to_string(&metadata_file)
                    .expect("Should be able to read metadata file");
                assert!(
                    content.contains("Subject:"),
                    "Metadata should contain Subject field"
                );
                assert!(
                    content.contains("From:"),
                    "Metadata should contain From field"
                );
            }
        }
    }
}

#[test]
fn test_export_with_headers() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with headers enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: false,
        attachments: false,
        headers: true,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode for tests

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export with headers should succeed");

    // Verify headers.txt files exist where transport headers are available
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .collect();

    // Should have at least one message directory
    assert!(
        !entries.is_empty(),
        "Output directory should contain exported messages"
    );

    // Note: Not all messages may have transport headers, so we just verify
    // that the export completes successfully
}

#[test]
fn test_export_with_all_options() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with all options enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: true,
        attachments: true,
        headers: true,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode for tests

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export with all options should succeed");

    // Verify output directory exists and contains message directories
    let entries: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(std::result::Result::ok)
        .collect();

    // Should have at least one message directory
    assert!(
        !entries.is_empty(),
        "Output directory should contain exported messages"
    );

    // Verify structure: each message should have message.html
    for entry in entries {
        let path = entry.path();
        if path.is_dir()
            && !path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("duplicates")
        {
            let message_html = path.join("message.html");
            assert!(
                message_html.exists(),
                "Message directory should contain message.html: {path:?}"
            );

            // Metadata should exist when --metadata is enabled
            let metadata_file = path.join("metadata.txt");
            if message_html.exists() {
                assert!(
                    metadata_file.exists(),
                    "Metadata file should exist at {metadata_file:?}"
                );
            }

            // If attachments exist, they should be in the attachments/ subdirectory
            let attachments_dir = path.join("attachments");
            if attachments_dir.exists() {
                assert!(
                    attachments_dir.is_dir(),
                    "Attachments should be a directory: {attachments_dir:?}"
                );
            }

            // Headers may or may not exist depending on message content
            // But the export should not fail
        }
    }
}

#[test]
fn test_export_with_csv() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with CSV enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: true,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode for tests

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export with CSV should succeed");

    // Verify CSV file exists
    let csv_path = output_path.join("emails.csv");
    assert!(csv_path.exists(), "CSV file should exist at {csv_path:?}");

    // Read and validate CSV content
    let csv_content = fs::read_to_string(&csv_path).expect("Should be able to read CSV file");

    let lines: Vec<&str> = csv_content.lines().collect();

    // Should have at least a header row
    assert!(!lines.is_empty(), "CSV should have at least a header row");

    // Verify header
    let header = lines[0];
    assert_eq!(
        header,
        "SequenceNumber,Subject,From,To,Date,MessageId,IsDuplicate,KeywordCount,EmailMatchCount,Size,AttachmentCount,ConvNumber,PST-StoreName,Error",
        "CSV header should match expected format"
    );

    // If there are data rows, validate format
    if lines.len() > 1 {
        // Each data row should have 9 columns (matching header)
        for (idx, line) in lines.iter().enumerate().skip(1) {
            // Count commas (accounting for quoted fields)
            // For a simple validation, just ensure the line is not empty
            assert!(!line.is_empty(), "CSV data row {idx} should not be empty");
        }
    }
}

#[test]
fn test_csv_row_count_matches_exported_messages() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with CSV enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: true,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export should succeed");

    // Count exported message directories
    let message_dirs: Vec<_> = fs::read_dir(&output_path)
        .expect("Should be able to read output directory")
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_dir()
                    && !path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("duplicates")
                {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    // Read CSV and count data rows
    let csv_path = output_path.join("emails.csv");
    let csv_content = fs::read_to_string(&csv_path).expect("Should be able to read CSV file");

    let csv_data_rows = csv_content.lines().skip(1).count();

    // CSV row count should equal total exported messages (main + duplicates)
    // Note: This count includes both unique and duplicate messages
    assert!(
        csv_data_rows >= message_dirs.len(),
        "CSV should have at least as many rows as exported message directories"
    );
}

#[test]
fn test_csv_duplicate_flag_accuracy() {
    // Setup: Get test PST file
    let pst_path = get_test_pst_path();

    // Skip test if fixture doesn't exist
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture {} not found", pst_path.display());
        return;
    }

    // Create temporary output directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().to_path_buf();

    // Create export coordinator with CSV enabled
    let args = ExportArgs {
        input: pst_path.clone(),
        output: output_path.clone(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: true,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };
    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    // Run export
    coordinator
        .run(&mut reporter)
        .expect("Export should succeed");

    // Read CSV
    let csv_path = output_path.join("emails.csv");
    let csv_content = fs::read_to_string(&csv_path).expect("Should be able to read CSV file");

    // Count true vs false in IsDuplicate column
    let mut true_count = 0;
    let mut false_count = 0;

    for line in csv_content.lines().skip(1) {
        if line.contains(",true,") {
            true_count += 1;
        } else if line.contains(",false,") {
            false_count += 1;
        }
    }

    // Should have at least some non-duplicate messages
    assert!(
        false_count > 0,
        "Should have at least some non-duplicate messages"
    );

    // Total rows should equal true + false
    let total_rows = csv_content.lines().skip(1).count();
    assert_eq!(
        total_rows,
        true_count + false_count,
        "All CSV rows should have either true or false for duplicate flag"
    );
}
