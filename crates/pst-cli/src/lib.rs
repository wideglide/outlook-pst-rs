//! PST CLI library
//!
//! Core library for the pst-cli command-line tool. Provides abstractions for
//! PST file processing, message extraction, and export coordination.

pub mod cli;
pub mod duplicate;
pub mod error;
pub mod export;
pub mod filter;
pub mod list;

use std::path::{Path, PathBuf};

use error::Result;

/// Represents a PST file source being processed
#[derive(Debug, Clone)]
pub struct PstFileSource {
    /// Path to the PST file
    pub path: PathBuf,
    /// Total number of messages (populated after scanning)
    pub message_count: Option<usize>,
}

impl PstFileSource {
    /// Create a new PST file source from a path
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            message_count: None,
        }
    }

    /// Check if the path exists and is a valid PST file
    pub fn validate(&self) -> Result<()> {
        if !self.path.exists() {
            return Err(error::Error::pst_not_found(self.path.clone()));
        }

        if !self.path.is_file() {
            return Err(error::Error::pst_invalid(
                self.path.clone(),
                "Path is not a file",
            ));
        }

        // Check file extension (case-sensitive on Linux/macOS)
        if let Some(ext) = self.path.extension() {
            if ext != "pst" {
                return Err(error::Error::pst_invalid(
                    self.path.clone(),
                    format!("Invalid extension: expected .pst, got .{}", ext.to_string_lossy()),
                ));
            }
        } else {
            return Err(error::Error::pst_invalid(
                self.path.clone(),
                "No file extension",
            ));
        }

        Ok(())
    }
}

/// Represents an email message extracted from a PST file
#[derive(Debug, Clone)]
pub struct PstMessage {
    /// Source PST file ID (for tracking across multiple files)
    pub pst_id: usize,
    /// Message subject
    pub subject: Option<String>,
    /// Sender email address
    pub from: Option<EmailAddress>,
    /// Recipient email addresses
    pub to: Vec<EmailAddress>,
    /// CC recipients
    pub cc: Vec<EmailAddress>,
    /// BCC recipients
    pub bcc: Vec<EmailAddress>,
    /// Message date/time
    pub date: Option<String>,
    /// Message-ID header
    pub message_id: Option<String>,
    /// HTML body (if available)
    pub body_html: Option<String>,
    /// RTF body (compressed, if available)
    pub body_rtf: Option<Vec<u8>>,
    /// Plain text body (if available)
    pub body_text: Option<String>,
    /// List of attachments
    pub attachments: Vec<Attachment>,
    /// Full message headers
    pub headers: Option<String>,
    /// Internal PST folder path
    pub folder_path: Option<String>,
    /// Message flags
    pub flags: Option<String>,
    /// Message size in bytes
    pub size: Option<usize>,
}

/// Email address with display name
#[derive(Debug, Clone)]
pub struct EmailAddress {
    /// Display name (e.g., "John Doe")
    pub display_name: Option<String>,
    /// Email address (e.g., "john@example.com")
    pub email_address: String,
}

impl EmailAddress {
    /// Create a new email address
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            display_name: None,
            email_address: email.into(),
        }
    }

    /// Create an email address with display name
    pub fn with_display_name(email: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            display_name: Some(display.into()),
            email_address: email.into(),
        }
    }

    /// Format for display (e.g., "John Doe <john@example.com>" or "john@example.com")
    pub fn format_display(&self) -> String {
        if let Some(ref name) = self.display_name {
            format!("{} <{}>", name, self.email_address)
        } else {
            self.email_address.clone()
        }
    }
}

/// Message attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Original filename
    pub filename: String,
    /// File size in bytes
    pub size: usize,
    /// Attachment data
    pub data: Vec<u8>,
}

/// Represents an exported message with metadata
#[derive(Debug)]
pub struct ExportItem {
    /// Sequential number (00001, 00002, etc.)
    pub sequence_number: u32,
    /// Output subdirectory path
    pub output_dir: PathBuf,
    /// Whether this message is a duplicate
    pub is_duplicate: bool,
    /// Matched keywords (if keyword filtering enabled)
    pub matched_keywords: Vec<String>,
    /// Matched email addresses (if email filtering enabled)
    pub matched_emails: Vec<String>,
}
