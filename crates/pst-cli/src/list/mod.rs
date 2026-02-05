//! List command implementation

use crate::cli::ListArgs;
use crate::error::Result;

/// List command for displaying PST folder structure
#[derive(Debug)]
pub struct ListCommand {
    /// List arguments from CLI
    args: ListArgs,
}

impl ListCommand {
    /// Create a new list command
    pub fn new(args: ListArgs) -> Self {
        Self { args }
    }

    /// Run the list command
    pub fn run(&self) -> Result<()> {
        // Validate PST file exists
        if !self.args.pst_file.exists() {
            return Err(crate::error::Error::pst_not_found(&self.args.pst_file));
        }

        // TODO: Implement in US8 (Phase 10)
        // This will:
        // - Use pst crate API to walk folder tree
        // - Count messages per folder without loading content
        // - Display tree-style output with indentation
        // - Show total counts

        Ok(())
    }
}
