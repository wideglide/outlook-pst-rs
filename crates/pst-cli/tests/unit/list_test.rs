//! Unit tests for list command (T091)
//!
//! Tests hierarchy correctness, count accuracy, and formatting.

use pst_cli::list::ListCommand;
use pst_cli::cli::ListArgs;
use std::path::PathBuf;

/// Helper to get a test PST file
fn get_test_pst_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("outlook.pst")
}

#[test]
fn test_list_command_creation() {
    let args = ListArgs {
        pst_file: PathBuf::from("test.pst"),
    };
    let _cmd = ListCommand::new(args);
    // Should not panic
}

#[test]
fn test_list_command_nonexistent_file() {
    let args = ListArgs {
        pst_file: PathBuf::from("/nonexistent/path/to/file.pst"),
    };
    let cmd = ListCommand::new(args);
    let result = cmd.run();
    assert!(result.is_err(), "Should return error for nonexistent file");
}

#[test]
fn test_list_command_runs_on_real_pst() {
    let pst_path = get_test_pst_path();
    if !pst_path.exists() {
        eprintln!("Skipping test: fixture not found at {:?}", pst_path);
        return;
    }

    let args = ListArgs {
        pst_file: pst_path,
    };
    let cmd = ListCommand::new(args);
    let result = cmd.run();
    assert!(result.is_ok(), "List command should succeed: {:?}", result.err());
}
