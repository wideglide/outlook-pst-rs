//! CLI argument parsing and command structures
//!
//! Uses clap derive API for type-safe command-line argument parsing.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod progress;

/// PST CLI Tool - Export and analyze PST files for eDiscovery
#[derive(Parser, Debug)]
#[command(name = "pst-cli")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Suppress progress indicators and summary statistics
    #[arg(long, global = true)]
    pub quiet: bool,

    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Export PST messages to HTML
    Export(ExportArgs),
    /// List PST folder structure
    List(ListArgs),
}

/// Arguments for the export subcommand
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ExportArgs {
    /// Path to PST file or directory containing PST files
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Output directory for exported messages
    #[arg(short = 'o', long = "output", value_name = "DIR")]
    pub output: PathBuf,

    /// Export metadata.txt with Subject,From, Date, etc.
    #[arg(long)]
    pub metadata: bool,

    /// Save email attachments
    #[arg(long)]
    pub attachments: bool,

    /// Export headers.txt with full transport headers
    #[arg(long)]
    pub headers: bool,

    /// Generate emails.csv summary spreadsheet
    #[arg(long)]
    pub csv: bool,

    /// Include draft (unsent) messages in export output
    #[arg(long)]
    pub drafts: bool,

    /// Comma-separated keywords to search for (case-insensitive)
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub keywords: Option<Vec<String>>,

    /// Comma-separated email addresses to search for (case-insensitive)
    #[arg(long, value_name = "LIST", value_delimiter = ',')]
    pub emails: Option<Vec<String>>,
}

/// Arguments for the list subcommand
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Path to PST file
    #[arg(value_name = "PST_FILE")]
    pub pst_file: PathBuf,
}

impl ExportArgs {
    /// Check if any filtering is enabled
    #[must_use] 
    pub fn has_filters(&self) -> bool {
        self.keywords.is_some() || self.emails.is_some()
    }

    /// Get keywords as normalized (lowercase, trimmed) list
    #[must_use] 
    pub fn normalized_keywords(&self) -> Option<Vec<String>> {
        self.keywords.as_ref().map(|kw_list| {
            kw_list
                .iter()
                .map(|kw| kw.trim().to_lowercase())
                .filter(|kw| !kw.is_empty())
                .collect()
        })
    }

    /// Get emails as normalized (lowercase, trimmed) list
    #[must_use] 
    pub fn normalized_emails(&self) -> Option<Vec<String>> {
        self.emails.as_ref().map(|email_list| {
            email_list
                .iter()
                .map(|email| email.trim().to_lowercase())
                .filter(|email| !email.is_empty())
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_keywords() {
        let args = ExportArgs {
            input: PathBuf::from("test.pst"),
            output: PathBuf::from("output"),
            metadata: false,
            attachments: false,
            headers: false,
            csv: false,
            drafts: false,
            keywords: Some(vec![
                "  Confidential  ".to_string(),
                "MERGER".to_string(),
                "".to_string(),
            ]),
            emails: None,
        };

        let normalized = args.normalized_keywords().unwrap();
        assert_eq!(normalized, vec!["confidential", "merger"]);
    }

    #[test]
    fn test_normalized_emails() {
        let args = ExportArgs {
            input: PathBuf::from("test.pst"),
            output: PathBuf::from("output"),
            metadata: false,
            attachments: false,
            headers: false,
            csv: false,
            drafts: false,
            keywords: None,
            emails: Some(vec![
                "  John@Example.COM  ".to_string(),
                "jane@example.com".to_string(),
            ]),
        };

        let normalized = args.normalized_emails().unwrap();
        assert_eq!(normalized, vec!["john@example.com", "jane@example.com"]);
    }
}
