//! Error types for pst-cli
//!
//! Defines comprehensive error types following the M-APP-ERROR principle.
//! All errors implement Display with actionable context and suggestions.

use std::path::PathBuf;
use std::{fmt, io};

/// Result type alias for pst-cli operations
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for pst-cli
#[derive(Debug)]
pub enum Error {
    /// PST file errors
    Pst(PstError),
    /// Export operation errors
    Export(ExportError),
    /// Duplicate detection errors
    Duplicate(DuplicateError),
    /// Filtering errors
    Filter(FilterError),
    /// I/O errors
    Io(io::Error),
    /// Other errors
    Other(anyhow::Error),
}

/// PST file-related errors
#[derive(Debug)]
pub enum PstError {
    /// PST file not found
    NotFound(PathBuf),
    /// PST file invalid or corrupted
    Invalid(PathBuf, String),
    /// PST parsing error
    ParseError(String),
}

/// Export operation errors
#[derive(Debug)]
pub enum ExportError {
    /// Output directory not writable
    OutputNotWritable(PathBuf),
    /// Output directory already exists with conflicts
    OutputConflict(PathBuf),
    /// Message export failed
    MessageFailed(u32, String),
    /// HTML conversion failed
    HtmlConversionFailed(String),
}

/// Duplicate detection errors
#[derive(Debug)]
pub enum DuplicateError {
    /// Hash generation failed
    HashFailed(String),
}

/// Filtering errors
#[derive(Debug)]
pub enum FilterError {
    /// Invalid keyword format
    InvalidKeyword(String),
    /// Invalid email format
    InvalidEmail(String),
}

// Implement Display for all error types with actionable messages
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Pst(e) => write!(f, "PST error: {}", e),
            Error::Export(e) => write!(f, "Export error: {}", e),
            Error::Duplicate(e) => write!(f, "Duplicate detection error: {}", e),
            Error::Filter(e) => write!(f, "Filter error: {}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Display for PstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PstError::NotFound(path) => {
                write!(
                    f,
                    "PST file not found: {}\n  Suggestion: Check the file path and ensure the file exists",
                    path.display()
                )
            }
            PstError::Invalid(path, reason) => {
                write!(
                    f,
                    "Invalid PST file: {}\n  Reason: {}\n  Suggestion: Ensure the file is a valid Outlook PST file",
                    path.display(),
                    reason
                )
            }
            PstError::ParseError(msg) => {
                write!(
                    f,
                    "Failed to parse PST: {}\n  Suggestion: The PST file may be corrupted or use an unsupported format",
                    msg
                )
            }
        }
    }
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::OutputNotWritable(path) => {
                write!(
                    f,
                    "Output directory not writable: {}\n  Suggestion: Check permissions or create the directory manually",
                    path.display()
                )
            }
            ExportError::OutputConflict(path) => {
                write!(
                    f,
                    "Output directory already contains numbered folders: {}\n  Suggestion: Use a different output directory or remove existing exports",
                    path.display()
                )
            }
            ExportError::MessageFailed(seq, msg) => {
                write!(
                    f,
                    "Failed to export message {}: {}\n  Note: Export will continue with remaining messages",
                    seq, msg
                )
            }
            ExportError::HtmlConversionFailed(msg) => {
                write!(
                    f,
                    "HTML conversion failed: {}\n  Note: Partial content may be exported",
                    msg
                )
            }
        }
    }
}

impl fmt::Display for DuplicateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuplicateError::HashFailed(msg) => {
                write!(f, "Content hash generation failed: {}", msg)
            }
        }
    }
}

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterError::InvalidKeyword(kw) => {
                write!(
                    f,
                    "Invalid keyword: '{}'\n  Suggestion: Use alphanumeric keywords separated by commas",
                    kw
                )
            }
            FilterError::InvalidEmail(email) => {
                write!(
                    f,
                    "Invalid email address: '{}'\n  Suggestion: Use valid email format: user@domain.com",
                    email
                )
            }
        }
    }
}

impl std::error::Error for Error {}
impl std::error::Error for PstError {}
impl std::error::Error for ExportError {}
impl std::error::Error for DuplicateError {}
impl std::error::Error for FilterError {}

// Conversions from other error types
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err)
    }
}

impl From<PstError> for Error {
    fn from(err: PstError) -> Self {
        Error::Pst(err)
    }
}

impl From<ExportError> for Error {
    fn from(err: ExportError) -> Self {
        Error::Export(err)
    }
}

impl From<DuplicateError> for Error {
    fn from(err: DuplicateError) -> Self {
        Error::Duplicate(err)
    }
}

impl From<FilterError> for Error {
    fn from(err: FilterError) -> Self {
        Error::Filter(err)
    }
}

// Convenience constructors
impl Error {
    /// Create a PST not found error
    pub fn pst_not_found(path: impl Into<PathBuf>) -> Self {
        Error::Pst(PstError::NotFound(path.into()))
    }

    /// Create a PST invalid error
    pub fn pst_invalid(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Error::Pst(PstError::Invalid(path.into(), reason.into()))
    }

    /// Create an output not writable error
    pub fn output_not_writable(path: impl Into<PathBuf>) -> Self {
        Error::Export(ExportError::OutputNotWritable(path.into()))
    }
}

// Re-export specific error types for convenience
pub use DuplicateError as Duplicate;
pub use ExportError as Export;
pub use FilterError as Filter;
pub use PstError as Pst;
