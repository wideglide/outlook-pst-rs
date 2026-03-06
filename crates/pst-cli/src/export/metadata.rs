//! Metadata extraction and formatting
//!
//! Handles extracting and formatting message metadata to metadata.txt files.

use crate::export::exporter::MessageData;
use crate::error::Result;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Extract and format metadata from a message
#[must_use] 
pub fn format_metadata(
    message: &MessageData,
    keywords_found: &[String],
    emails_found: &[String],
) -> String {
    let mut output = String::new();

    // Subject
    let _ = writeln!(output, "Subject: {}", message.subject);

    // From
    let _ = writeln!(output, "From: {}", message.from);

    // To
    if message.to.is_empty() {
        output.push_str("To: N/A\n");
    } else {
        let _ = writeln!(output, "To: {}", message.to.join("; "));
    }

    // CC
    if message.cc.is_empty() {
        output.push_str("CC: N/A\n");
    } else {
        let _ = writeln!(output, "CC: {}", message.cc.join("; "));
    }

    // BCC
    if message.bcc.is_empty() {
        output.push_str("BCC: N/A\n");
    } else {
        let _ = writeln!(output, "BCC: {}", message.bcc.join("; "));
    }

    // Date
    let _ = writeln!(output, "Date: {}", message.date);

    // Message-ID
    let _ = writeln!(
        output,
        "Message-ID: {}",
        message.message_id.as_deref().unwrap_or("N/A")
    );

    // Folder
    let _ = writeln!(output, "Folder: {}", message.folder_path);

    // Size
    if let Some(size) = message.size_bytes {
        let _ = writeln!(output, "Size: {size} bytes");
    } else {
        output.push_str("Size: N/A\n");
    }

    // Attachments
    if message.attachments.is_empty() {
        output.push_str("Attachments: none\n");
    } else {
        let attachment_names: Vec<&str> = message
            .attachments
            .iter()
            .map(|a| a.filename.as_str())
            .collect();
        let _ = writeln!(
            output,
            "Attachments: {} (files: {})",
            attachment_names.len(),
            attachment_names.join(", ")
        );
    }

    // Flags
    if message.flags.is_empty() {
        output.push_str("Flags: none\n");
    } else {
        let _ = writeln!(output, "Flags: {}", message.flags.join(", "));
    }

    // Keywords (if any)
    if keywords_found.is_empty() {
        output.push_str("Keywords: none\n");
    } else {
        let _ = writeln!(output, "Keywords: {}", keywords_found.join(", "));
    }

    // Email matches (if any)
    if emails_found.is_empty() {
        output.push_str("Email Matches: none\n");
    } else {
        let _ = writeln!(output, "Email Matches: {}", emails_found.join(", "));
    }

    output
}

/// Write metadata file to disk
///
/// # Errors
///
/// Returns an error if the file cannot be written to disk.
pub fn write_metadata_file(
    output_path: &Path,
    metadata_content: &str,
) -> Result<()> {
    fs::write(output_path, metadata_content).map_err(|e| {
        crate::error::Error::Export(
            crate::error::ExportError::MessageFailed(
                0,
                format!("Failed to write metadata.txt: {e}"),
            ),
        )
    })
}

/// Sanitize filename for filesystem safety
/// Replaces unsafe characters with underscores
#[must_use] 
pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = String::new();
    
    for ch in filename.chars() {
        match ch {
            // Unsafe filesystem characters
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => {
                sanitized.push('_');
            }
            // Control characters
            ch if ch.is_control() => {
                sanitized.push('_');
            }
            // Safe characters
            _ => {
                sanitized.push(ch);
            }
        }
    }

    // Trim leading/trailing dots and spaces (unsafe on some filesystems)
    sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();

    // If empty after sanitization, use default name
    if sanitized.is_empty() {
        sanitized = "attachment".to_string();
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_basic() {
        assert_eq!(sanitize_filename("document.pdf"), "document.pdf");
        assert_eq!(sanitize_filename("my file.txt"), "my file.txt");
    }

    #[test]
    fn test_sanitize_filename_unsafe_chars() {
        assert_eq!(sanitize_filename("file/path.txt"), "file_path.txt");
        assert_eq!(sanitize_filename("file\\path.txt"), "file_path.txt");
        assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("file*?.txt"), "file__.txt");
        assert_eq!(sanitize_filename("file<>name.txt"), "file__name.txt");
    }

    #[test]
    fn test_sanitize_filename_dots() {
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("file."), "file");
        assert_eq!(sanitize_filename("..."), "attachment");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "attachment");
        assert_eq!(sanitize_filename("   "), "attachment");
    }
}
