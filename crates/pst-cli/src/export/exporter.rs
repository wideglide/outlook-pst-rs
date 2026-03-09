//! PST message exporter
//!
//! Handles reading PST message data and writing exported HTML files.

use super::html::convert_to_html;
use super::metadata::{format_metadata, sanitize_filename};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Represents a message to be exported
#[derive(Debug, Clone)]
pub struct MessageData {
    /// Email subject line
    pub subject: String,
    /// Sender email address
    pub from: String,
    /// Primary recipients
    pub to: Vec<String>,
    /// Carbon copy recipients
    pub cc: Vec<String>,
    /// Blind carbon copy recipients
    pub bcc: Vec<String>,
    /// Message date
    pub date: String,
    /// Message-ID header
    pub message_id: Option<String>,
    /// HTML body content
    pub body_html: Option<String>,
    /// Compressed RTF body
    pub body_rtf: Option<Vec<u8>>,
    /// Plain text body
    pub body_plain: Option<String>,
    /// Internal folder path
    pub folder_path: String,
    /// Message attachments
    pub attachments: Vec<Attachment>,
    /// Full message headers
    pub headers: Option<String>,
    /// Message size in bytes
    pub size_bytes: Option<u64>,
    /// Message flags (read, flagged, etc.)
    pub flags: Vec<String>,
    /// Whether this message is a draft/unsent item
    pub is_draft: bool,
    /// Canonical conversation id (`PidTagConversationId`, 16 bytes) when present
    pub conversation_id: Option<Vec<u8>>,
    /// Raw conversation index bytes (`PidTagConversationIndex`) when present
    pub conversation_index: Option<Vec<u8>>,
}

/// Represents an email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Attachment filename
    pub filename: String,
    /// Attachment binary data
    pub data: Vec<u8>,
    /// MIME type (if available)
    pub content_type: Option<String>,
}

impl MessageData {
    /// Create an example message for testing
    #[cfg(test)]
    #[must_use]
    pub fn example() -> Self {
        Self {
            subject: "Test Message".to_string(),
            from: "sender@example.com".to_string(),
            to: vec!["recipient@example.com".to_string()],
            cc: vec![],
            bcc: vec![],
            date: "2026-02-05T10:00:00Z".to_string(),
            message_id: Some("test@example.com".to_string()),
            body_html: Some("<p>This is a test message</p>".to_string()),
            body_rtf: None,
            body_plain: None,
            folder_path: "Inbox".to_string(),
            attachments: vec![],
            headers: None,
            size_bytes: Some(1024),
            flags: vec![],
            is_draft: false,
            conversation_id: None,
            conversation_index: None,
        }
    }
}

/// Exporter for writing message files to disk
pub struct MessageExporter {
    /// Base output directory
    pub output_dir: PathBuf,
}

impl MessageExporter {
    /// Create a new message exporter
    #[must_use]
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Export a message to the specified numbered directory
    /// If `is_duplicate` is true, exports to duplicates/ subdirectory
    ///
    /// # Errors
    ///
    /// Returns an error if the output directory cannot be created or the HTML file cannot be written.
    pub fn export_message(
        &self,
        message: &MessageData,
        sequence_number: u32,
        is_duplicate: bool,
        conversation_folder: Option<&str>,
    ) -> Result<PathBuf> {
        let message_dir = self.message_dir(sequence_number, is_duplicate, conversation_folder);

        // Create directory
        fs::create_dir_all(&message_dir).map_err(|_e| {
            crate::error::Error::Export(crate::error::ExportError::OutputConflict(
                message_dir.clone(),
            ))
        })?;

        // Convert body to HTML
        let body_html = convert_to_html(
            message.body_html.as_deref(),
            message.body_rtf.as_deref(),
            message.body_plain.as_deref(),
            None,
        )
        .unwrap_or_else(|_| {
            r"<html><body><p>Error converting message body</p></body></html>".to_string()
        });

        // Build email header block
        let html_content = inject_message_header(&body_html, message);

        // Write message.html file
        let html_path = message_dir.join("message.html");
        fs::write(&html_path, html_content).map_err(|e| {
            crate::error::Error::Export(crate::error::ExportError::MessageFailed(
                sequence_number,
                format!("Failed to write message.html: {e}"),
            ))
        })?;

        Ok(html_path)
    }

    /// Write metadata file for a message (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata file cannot be written.
    pub fn write_metadata(
        &self,
        message: &MessageData,
        sequence_number: u32,
        is_duplicate: bool,
        conversation_folder: Option<&str>,
        keywords_found: &[String],
        emails_found: &[String],
    ) -> Result<()> {
        let metadata_path = self
            .message_dir(sequence_number, is_duplicate, conversation_folder)
            .join("metadata.txt");

        let metadata_content = format_metadata(message, keywords_found, emails_found);

        fs::write(&metadata_path, metadata_content).map_err(|e| {
            crate::error::Error::Export(crate::error::ExportError::MessageFailed(
                sequence_number,
                format!("Failed to write metadata: {e}"),
            ))
        })?;

        Ok(())
    }

    /// Export attachments for a message (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if an attachment file cannot be written.
    pub fn write_attachments(
        &self,
        message: &MessageData,
        sequence_number: u32,
        is_duplicate: bool,
        conversation_folder: Option<&str>,
    ) -> Result<()> {
        if message.attachments.is_empty() {
            return Ok(());
        }

        let message_dir = self.message_dir(sequence_number, is_duplicate, conversation_folder);

        // Track filename collisions
        let mut used_filenames: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for attachment in &message.attachments {
            let mut filename = sanitize_filename(&attachment.filename);

            // Handle filename collisions by adding numeric suffix
            if let Some(count) = used_filenames.get_mut(&filename) {
                *count += 1;
                // Split filename and extension
                if let Some(dot_pos) = filename.rfind('.') {
                    let (name, ext) = filename.split_at(dot_pos);
                    filename = format!("{name}_{count}{ext}");
                } else {
                    filename = format!("{filename}_{count}");
                }
            }
            used_filenames.insert(filename.clone(), 1);

            // Write attachment file
            let attachment_path = message_dir.join(&filename);
            fs::write(&attachment_path, &attachment.data).map_err(|e| {
                crate::error::Error::Export(crate::error::ExportError::MessageFailed(
                    sequence_number,
                    format!("Failed to write attachment {filename}: {e}"),
                ))
            })?;
        }

        Ok(())
    }

    /// Write message headers to headers.txt (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if the headers file cannot be written.
    pub fn write_headers(
        &self,
        message: &MessageData,
        sequence_number: u32,
        is_duplicate: bool,
        conversation_folder: Option<&str>,
    ) -> Result<()> {
        let headers_path = self
            .message_dir(sequence_number, is_duplicate, conversation_folder)
            .join("headers.txt");

        let headers_content = message.headers.as_deref().unwrap_or("No headers available");

        fs::write(&headers_path, headers_content).map_err(|e| {
            crate::error::Error::Export(crate::error::ExportError::MessageFailed(
                sequence_number,
                format!("Failed to write headers: {e}"),
            ))
        })?;

        Ok(())
    }

    fn message_dir(
        &self,
        sequence_number: u32,
        is_duplicate: bool,
        conversation_folder: Option<&str>,
    ) -> PathBuf {
        let seq_str = format!("{sequence_number:05}");
        let mut dir = self.output_dir.clone();

        if is_duplicate {
            dir.push("duplicates");
        }

        if let Some(folder) = conversation_folder {
            dir.push(folder);
        }

        dir.push(seq_str);
        dir
    }
}

/// Build an HTML header block with message metadata and inject it after `<body>`.
fn inject_message_header(html: &str, message: &MessageData) -> String {
    use html_escape::encode_text;
    use std::fmt::Write;

    let mut header = String::from(
        r#"<div style="font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Arial,sans-serif;border-bottom:1px solid #ccc;padding-bottom:8px;margin-bottom:12px;">"#,
    );

    let _ = writeln!(
        header,
        "<b>Subject:</b> {}<br>",
        encode_text(&message.subject)
    );
    let _ = writeln!(header, "<b>From:</b> {}<br>", encode_text(&message.from));
    let _ = writeln!(header, "<b>Date:</b> {}<br>", encode_text(&message.date));
    let _ = writeln!(
        header,
        "<b>To:</b> {}<br>",
        encode_text(&message.to.join("; "))
    );

    if !message.cc.is_empty() {
        let _ = writeln!(
            header,
            "<b>CC:</b> {}<br>",
            encode_text(&message.cc.join("; "))
        );
    }
    if !message.bcc.is_empty() {
        let _ = writeln!(
            header,
            "<b>BCC:</b> {}<br>",
            encode_text(&message.bcc.join("; "))
        );
    }

    header.push_str("</div>\n");

    // Insert after <body> tag (case-insensitive search)
    let lower = html.to_ascii_lowercase();
    if let Some(pos) = lower.find("<body") {
        // Find the closing '>' of the <body ...> tag
        if let Some(end) = html[pos..].find('>') {
            let insert_at = pos + end + 1;
            let mut result = String::with_capacity(html.len() + header.len());
            result.push_str(&html[..insert_at]);
            result.push('\n');
            result.push_str(&header);
            result.push_str(&html[insert_at..]);
            return result;
        }
    }

    // Fallback: prepend header inside a wrapper
    format!(
        r#"<html><head><meta charset="utf-8"></head><body>
{header}{html}</body></html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_export_message_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MessageExporter::new(temp_dir.path().to_path_buf());
        let message = MessageData::example();

        let result = exporter.export_message(&message, 1, false, None);
        assert!(result.is_ok());

        let message_dir = temp_dir.path().join("00001");
        assert!(message_dir.exists());
    }

    #[test]
    fn test_export_message_writes_html() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MessageExporter::new(temp_dir.path().to_path_buf());
        let message = MessageData::example();

        exporter.export_message(&message, 1, false, None).unwrap();

        let html_path = temp_dir.path().join("00001").join("message.html");
        assert!(html_path.exists());

        let content = fs::read_to_string(&html_path).unwrap();
        assert!(content.contains("test message") || content.contains("Test"));
    }

    #[test]
    fn test_write_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let exporter = MessageExporter::new(temp_dir.path().to_path_buf());
        let message = MessageData::example();

        exporter.export_message(&message, 1, false, None).unwrap();
        exporter
            .write_metadata(
                &message,
                1,
                false,
                None,
                &["keyword1".to_string()],
                &["test@example.com".to_string()],
            )
            .unwrap();

        let metadata_path = temp_dir.path().join("00001").join("metadata.txt");
        assert!(metadata_path.exists());

        let content = fs::read_to_string(&metadata_path).unwrap();
        assert!(content.contains("Test Message"));
        assert!(content.contains("keyword1"));
    }
}
