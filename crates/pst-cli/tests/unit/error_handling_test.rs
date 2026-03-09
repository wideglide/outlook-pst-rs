use pst_cli::error::{Error, PstError};

#[test]
fn test_error_display_pst_not_found() {
    let error = Error::pst_not_found("/nonexistent/file.pst");
    let display = format!("{error}");

    assert!(display.contains("PST file not found"));
    assert!(display.contains("/nonexistent/file.pst"));
    assert!(display.contains("Suggestion"));
}

#[test]
fn test_error_display_output_not_writable() {
    let error = Error::output_not_writable("/readonly/dir");
    let display = format!("{error}");

    assert!(display.contains("not writable"));
    assert!(display.contains("/readonly/dir"));
}

#[test]
fn test_pst_error_invalid() {
    let error = PstError::Invalid("/test.xyz".into(), "Invalid extension".to_string());
    let display = format!("{error}");

    assert!(display.contains("Invalid PST file"));
    assert!(display.contains("Invalid extension"));
}

#[test]
fn test_error_conversion_from_io() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error: pst_cli::error::Error = io_error.into();
    let display = format!("{error}");

    assert!(display.contains("I/O error"));
}
