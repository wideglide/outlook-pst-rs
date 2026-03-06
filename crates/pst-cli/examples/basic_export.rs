//! Basic PST Export Example
//!
//! Demonstrates how to export messages from a single PST file to HTML format.
//! This example shows the simplest use case with progress reporting.
//!
//! Usage:
//! ```bash
//! cargo run --example basic_export -- <pst-file> <output-directory>
//! ```
//!
//! Example:
//! ```bash
//! cargo run --example basic_export -- my_archive.pst ./export_output
//! ```

use pst_cli::cli::ExportArgs;
use pst_cli::export::ExportCoordinator;
use pst_cli::cli::progress::ProgressReporter;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <pst-file> <output-directory>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} archive.pst ./my_export", args[0]);
        eprintln!();
        eprintln!("Description:");
        eprintln!("  Exports all messages from <pst-file> to HTML format in <output-directory>.");
        eprintln!("  Each message is exported to a numbered subdirectory (00001/, 00002/, etc.)");
        eprintln!("  with a message.html file containing the message content.");
        process::exit(1);
    }
    
    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);
    
    // Validate input file exists
    if !input_path.exists() {
        eprintln!("Error: PST file not found: {}", input_path.display());
        eprintln!("Please provide a valid path to a .pst file");
        process::exit(2);
    }
    
    // Validate input has .pst extension
    if let Some(ext) = input_path.extension() {
        if ext.to_str().unwrap_or("").to_lowercase() != "pst" {
            eprintln!("Warning: File does not have .pst extension: {}", input_path.display());
            eprintln!("Attempting to process anyway...");
        }
    }
    
    // Create export arguments with basic settings
    let export_args = ExportArgs {
        input: input_path.clone(),
        output: output_path.clone(),
        metadata: false,       // Not exporting metadata.txt
        attachments: false,    // Not exporting attachments
        headers: false,        // Not exporting headers.txt
        csv: false,           // Not generating CSV summary
        drafts: false,        // Skip drafts by default
        keywords: None,       // No keyword filtering
        emails: None,         // No email filtering
    };
    
    println!("PST Export Tool - Basic Example");
    println!("================================");
    println!();
    println!("Input:  {}", input_path.display());
    println!("Output: {}", output_path.display());
    println!();
    
    // Create export coordinator
    let mut coordinator = ExportCoordinator::new(export_args);
    
    // Create progress reporter (not quiet, will show progress)
    let mut reporter = ProgressReporter::new(false);
    
    println!("Starting export...");
    println!();
    
    // Run the export
    match coordinator.run(&mut reporter) {
        Ok(()) => {
            println!();
            println!("✅ Export completed successfully!");
            println!();
            println!("Messages exported to: {}", output_path.display());
            println!();
            println!("Each message is in a numbered directory:");
            println!("  00001/message.html");
            println!("  00002/message.html");
            println!("  ...");
            println!();
            println!("You can open any message.html file in a web browser to view the content.");
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ Export failed: {}", e);
            eprintln!();
            match &e {
                pst_cli::error::Error::Pst(pst_error) => {
                    eprintln!("PST Error Details:");
                    eprintln!("  {}", pst_error);
                    eprintln!();
                    eprintln!("Suggestions:");
                    eprintln!("  - Verify the PST file is not corrupted");
                    eprintln!("  - Check if the PST file is in use by another application");
                    eprintln!("  - Try a different PST file");
                }
                pst_cli::error::Error::Io(io_error) => {
                    eprintln!("I/O Error Details:");
                    eprintln!("  {}", io_error);
                    eprintln!();
                    eprintln!("Suggestions:");
                    eprintln!("  - Check file permissions");
                    eprintln!("  - Verify the output directory is writable");
                    eprintln!("  - Ensure sufficient disk space");
                }
                _ => {
                    eprintln!("Error Details:");
                    eprintln!("  {}", e);
                }
            }
            process::exit(3);
        }
    }
}
