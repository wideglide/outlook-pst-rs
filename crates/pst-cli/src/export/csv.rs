//! CSV export functionality for PST message metadata
//!
//! Generates a CSV summary file (emails.csv) with one row per processed message.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use crate::error::Result;

/// CSV exporter managing spreadsheet generation
pub struct CsvExporter {
    writer: BufWriter<File>,
    rows_written: usize,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for CsvExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CsvExporter")
            .field("rows_written", &self.rows_written)
            .finish()
    }
}

/// Data for a single CSV row
#[derive(Debug, Clone)]
pub struct CsvRow {
    /// Message sequence number (1-based, zero-padded to 5 digits on output)
    pub sequence_number: u32,
    /// Email subject line
    pub subject: String,
    /// Sender display name and/or address
    pub from: String,
    /// To recipients (comma-separated)
    pub to: String,
    /// Delivery date in ISO-like format
    pub date: String,
    /// Message-ID header value
    pub message_id: String,
    /// Whether this message is a duplicate of a previously seen message
    pub is_duplicate: bool,
    /// Number of distinct keywords matched in subject and body
    pub keyword_count: usize,
    /// Number of distinct email addresses matched in From/To/CC/BCC fields
    pub email_match_count: usize,
}

impl CsvExporter {
    /// Create a new CSV exporter, writing to the specified path
    ///
    /// # Errors
    ///
    /// Returns an error if the output file cannot be created.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer,
            rows_written: 0,
        })
    }

    /// Write the CSV header row
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure when writing to the underlying file.
    pub fn write_header(&mut self) -> Result<()> {
        writeln!(
            self.writer,
            "SequenceNumber,Subject,From,To,Date,MessageId,IsDuplicate,KeywordCount,EmailMatchCount"
        )?;
        Ok(())
    }

    /// Write a data row to the CSV
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure when writing to the underlying file.
    pub fn write_row(&mut self, row: &CsvRow) -> Result<()> {
        let line = format!(
            "{},{},{},{},{},{},{},{},{}",
            row.sequence_number,
            escape_csv_field(&row.subject),
            escape_csv_field(&row.from),
            escape_csv_field(&row.to),
            escape_csv_field(&row.date),
            escape_csv_field(&row.message_id),
            row.is_duplicate,
            row.keyword_count,
            row.email_match_count
        );
        writeln!(self.writer, "{line}")?;
        self.rows_written += 1;
        Ok(())
    }

    /// Get the number of rows written (excluding header)
    #[must_use] 
    pub fn rows_written(&self) -> usize {
        self.rows_written
    }

    /// Flush any buffered data to disk
    ///
    /// # Errors
    ///
    /// Returns an error on I/O failure when flushing the buffer.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// Escape a field for CSV format
///
/// Rules:
/// - If field contains comma, quote, or newline, wrap in quotes
/// - If field contains quotes, double them and wrap in quotes
/// - Otherwise, use field as-is
fn escape_csv_field(field: &str) -> String {
    // Check if escaping is needed
    let needs_quoting = field.contains(',') 
        || field.contains('"') 
        || field.contains('\n')
        || field.contains('\r');

    if needs_quoting {
        // Escape quotes by doubling them
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field_simple() {
        assert_eq!(escape_csv_field("simple text"), "simple text");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        assert_eq!(escape_csv_field("Hello, World"), "\"Hello, World\"");
    }

    #[test]
    fn test_escape_csv_field_with_quotes() {
        assert_eq!(escape_csv_field("He said \"hello\""), "\"He said \"\"hello\"\"\"");
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        assert_eq!(escape_csv_field("Line 1\nLine 2"), "\"Line 1\nLine 2\"");
    }

    #[test]
    fn test_escape_csv_field_with_carriage_return() {
        assert_eq!(escape_csv_field("Line 1\r\nLine 2"), "\"Line 1\r\nLine 2\"");
    }

    #[test]
    fn test_escape_csv_field_empty() {
        assert_eq!(escape_csv_field(""), "");
    }

    #[test]
    fn test_escape_csv_field_multiple_special_chars() {
        assert_eq!(
            escape_csv_field("Test, \"quoted\", value\nwith newline"),
            "\"Test, \"\"quoted\"\", value\nwith newline\""
        );
    }
}
