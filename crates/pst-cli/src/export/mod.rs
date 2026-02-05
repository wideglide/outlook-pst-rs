//! Export command orchestration and file I/O

use crate::cli::ExportArgs;
use crate::error::Result;
use std::path::PathBuf;

pub mod csv;
pub mod exporter;
pub mod html;
pub mod metadata;

/// Export coordinator managing the overall export workflow
#[derive(Debug)]
pub struct ExportCoordinator {
    /// Export arguments from CLI
    args: ExportArgs,
    /// Next sequence number for messages
    next_sequence: u32,
}

impl ExportCoordinator {
    /// Create a new export coordinator
    pub fn new(args: ExportArgs) -> Self {
        Self {
            args,
            next_sequence: 1,
        }
    }

    /// Get the next sequence number and increment counter
    pub fn next_sequence_number(&mut self) -> u32 {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        seq
    }

    /// Format a sequence number as zero-padded 5-digit string (00001, 00002, etc.)
    pub fn format_sequence(sequence: u32) -> String {
        format!("{:05}", sequence)
    }

    /// Get output directory for a message (main or duplicates/)
    pub fn get_message_output_dir(&self, sequence: u32, is_duplicate: bool) -> PathBuf {
        let seq_str = Self::format_sequence(sequence);
        let mut path = self.args.output.clone();
        
        if is_duplicate {
            path.push("duplicates");
        }
        
        path.push(seq_str);
        path
    }

    /// Run the export operation
    pub fn run(&mut self) -> Result<()> {
        // Validate input path exists
        if !self.args.input.exists() {
            return Err(crate::error::Error::pst_not_found(&self.args.input));
        }

        // Validate output directory is writable
        if self.args.output.exists() && !self.args.output.is_dir() {
            return Err(crate::error::Error::output_not_writable(&self.args.output));
        }

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.args.output)?;

        // TODO: Implement actual export logic in user story phases
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_formatting() {
        assert_eq!(ExportCoordinator::format_sequence(1), "00001");
        assert_eq!(ExportCoordinator::format_sequence(42), "00042");
        assert_eq!(ExportCoordinator::format_sequence(12345), "12345");
    }

    #[test]
    fn test_next_sequence_number() {
        let args = ExportArgs {
            input: PathBuf::from("test.pst"),
            output: PathBuf::from("output"),
            metadata: false,
            attachments: false,
            headers: false,
            csv: false,
            keywords: None,
            emails: None,
        };

        let mut coord = ExportCoordinator::new(args);
        assert_eq!(coord.next_sequence_number(), 1);
        assert_eq!(coord.next_sequence_number(), 2);
        assert_eq!(coord.next_sequence_number(), 3);
    }
}
