//! Integration tests for list command (T092)
//!
//! Tests list command output format and accuracy on sample PST files.

use std::path::PathBuf;
use std::process::Command;

/// Helper to get the path to pst-cli binary
fn get_binary_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("pst-cli")
}

/// Helper to get a test PST file
fn get_test_pst_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("outlook.pst")
}

#[test]
fn test_list_command_output_format() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    // Use the ListCommand directly
    let args = pst_cli::cli::ListArgs {
        pst_file: pst_path,
    };
    let cmd = pst_cli::list::ListCommand::new(args);
    let result = cmd.run();
    assert!(result.is_ok(), "List should succeed: {:?}", result.err());
}

#[test]
fn test_list_command_shows_folder_names() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    // Capture output by running command via binary
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping test: binary not built yet at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["list", pst_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute pst-cli list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should contain PST file info
    assert!(stdout.contains("PST File:"), "Output should mention PST File");
    
    // Should contain message count info
    assert!(stdout.contains("messages"), "Output should reference message counts");
    
    // Should contain total
    assert!(stdout.contains("Total messages"), "Output should show total message count");
}

#[test]
fn test_list_command_nonexistent_pst_error() {
    let args = pst_cli::cli::ListArgs {
        pst_file: PathBuf::from("/tmp/nonexistent_pst_file_12345.pst"),
    };
    let cmd = pst_cli::list::ListCommand::new(args);
    let result = cmd.run();
    assert!(result.is_err(), "Should fail for nonexistent PST file");
}
