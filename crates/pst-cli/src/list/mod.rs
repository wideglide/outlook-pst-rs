//! List command implementation

use crate::cli::ListArgs;
use crate::error::Result;
use outlook_pst::messaging::folder::Folder;
use outlook_pst::messaging::store::Store;
use outlook_pst::ndb::node_id::NodeId;
use std::rc::Rc;

/// List command for displaying PST folder structure
#[derive(Debug)]
pub struct ListCommand {
    /// List arguments from CLI
    args: ListArgs,
}

impl ListCommand {
    /// Create a new list command
    #[must_use]
    pub fn new(args: ListArgs) -> Self {
        Self { args }
    }

    /// Run the list command
    ///
    /// # Errors
    ///
    /// Returns an error if the PST file doesn't exist or cannot be opened.
    pub fn run(&self) -> Result<()> {
        // Validate PST file exists
        if !self.args.pst_file.exists() {
            return Err(crate::error::Error::pst_not_found(&self.args.pst_file));
        }

        // Open PST store
        let store = outlook_pst::open_store(&self.args.pst_file).map_err(|e| {
            crate::error::Error::Other(anyhow::anyhow!("Failed to open PST file: {e}"))
        })?;

        // Get properties to verify it's a valid store
        let properties = store.properties();
        let display_name = properties
            .display_name()
            .unwrap_or_else(|_| "[Unknown Store]".to_string());

        println!("\n📧 PST File: {}", self.args.pst_file.display());
        println!("Store Name: {display_name}");
        println!("├─ Folder Structure:\n");

        // Get IPM subtree entry and recursively list folders
        match properties.ipm_sub_tree_entry_id() {
            Ok(entry_id) => {
                if let Ok(root_folder) = store.open_folder(&entry_id) {
                    let mut total_messages = 0;
                    Self::list_folder_recursive(&store, &root_folder, "", &mut total_messages);
                    println!("\n✅ Total messages in PST: {total_messages}\n");
                } else {
                    println!("⚠️  Could not open root folder\n");
                }
            }
            Err(e) => {
                println!("⚠️  Could not access folder structure: {e}\n");
            }
        }

        Ok(())
    }

    /// Recursively list folders and count messages
    fn list_folder_recursive(
        store: &Rc<dyn Store>,
        folder: &Rc<dyn Folder>,
        indent: &str,
        total_messages: &mut usize,
    ) {
        // Get folder name
        let folder_name = folder
            .properties()
            .display_name()
            .unwrap_or_else(|_| "Unknown".to_string());

        // Count messages in this folder
        let message_count = if let Some(contents_table) = folder.contents_table() {
            let count = contents_table.rows_matrix().count();
            *total_messages += count;
            count
        } else {
            0
        };

        // Display folder with message count
        println!("{indent}📁 {folder_name} ({message_count} messages)");

        // Recursively process subfolders
        if let Some(hierarchy_table) = folder.hierarchy_table() {
            let mut subfolders = Vec::new();

            // Collect subfolders from hierarchy table
            for row in hierarchy_table.rows_matrix() {
                // Get the NodeId from the row ID
                let node_id = NodeId::from(u32::from(row.id()));

                // Convert to EntryId using store properties
                if let Ok(entry_id) = store.properties().make_entry_id(node_id) {
                    subfolders.push(entry_id);
                }
            }

            // Process subfolders
            let next_indent = format!("{indent}  ");
            for entry_id in subfolders {
                if let Ok(subfolder) = store.open_folder(&entry_id) {
                    Self::list_folder_recursive(store, &subfolder, &next_indent, total_messages);
                }
            }
        }
    }
}
