//! Batch PST Export Example
//!
//! Demonstrates how to export messages from multiple PST files in a directory.
//! PST files are processed in alphabetical order with sequential numbering
//! across all files. If any PST file fails to process, the export continues
//! with remaining files.
//!
//! Usage:
//! ```bash
//! cargo run --example batch_export -- <pst-directory> <output-directory>
//! ```
//!
//! Example:
//! ```bash
//! cargo run --example batch_export -- ./pst_files ./batch_export_output
//! ```

use pst_cli::cli::progress::ProgressReporter;
use pst_cli::cli::ExportArgs;
use pst_cli::export::ExportCoordinator;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

#[allow(clippy::too_many_lines)]
fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <pst-directory> <output-directory>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} ./pst_files ./my_batch_export", args[0]);
        eprintln!();
        eprintln!("Description:");
        eprintln!("  Exports all messages from ALL .pst files in <pst-directory> to HTML format.");
        eprintln!("  PST files are processed in alphabetical order (deterministic).");
        eprintln!("  Messages are numbered sequentially across all files (00001, 00002, ...).");
        eprintln!();
        eprintln!("Features:");
        eprintln!("  - Alphabetical PST file processing (alice.pst before bob.pst)");
        eprintln!("  - Continuous numbering across files");
        eprintln!("  - Error resilience: continues if individual PST files fail");
        eprintln!("  - Duplicate detection across all files");
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    // Validate input directory exists
    if !input_path.exists() {
        eprintln!("Error: Directory not found: {}", input_path.display());
        eprintln!("Please provide a valid path to a directory containing .pst files");
        process::exit(2);
    }

    // Validate input is a directory
    if !input_path.is_dir() {
        eprintln!("Error: Path is not a directory: {}", input_path.display());
        eprintln!("For single file export, use the basic_export example instead");
        process::exit(2);
    }

    // Count PST files in directory
    let pst_files: Vec<_> = match fs::read_dir(&input_path) {
        Ok(entries) => entries
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext.to_str().unwrap_or("").to_lowercase() == "pst")
            })
            .collect(),
        Err(e) => {
            eprintln!(
                "Error: Cannot read directory {}: {}",
                input_path.display(),
                e
            );
            process::exit(2);
        }
    };

    if pst_files.is_empty() {
        eprintln!("Error: No .pst files found in {}", input_path.display());
        eprintln!();
        eprintln!("Please ensure the directory contains at least one .pst file");
        process::exit(2);
    }

    // Sort PST files alphabetically to show processing order
    let mut pst_file_names: Vec<String> = pst_files
        .iter()
        .filter_map(|e| e.file_name().to_str().map(std::string::ToString::to_string))
        .collect();
    pst_file_names.sort();

    println!("PST Export Tool - Batch Export Example");
    println!("=======================================");
    println!();
    println!("Input Directory:  {}", input_path.display());
    println!("Output Directory: {}", output_path.display());
    println!();
    println!("Found {} PST file(s) to process:", pst_files.len());
    for (i, name) in pst_file_names.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();
    println!("Processing Order: Alphabetical (deterministic)");
    println!("Numbering: Sequential across all files");
    println!("Error Handling: Continue on individual file failures");
    println!();

    // Create export arguments
    let export_args = ExportArgs {
        input: input_path.clone(),
        output: output_path.clone(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    };

    // Create export coordinator
    let mut coordinator = ExportCoordinator::new(export_args);

    // Create progress reporter (not quiet)
    let mut reporter = ProgressReporter::new(false);

    println!("Starting batch export...");
    println!();

    // Run the export
    match coordinator.run(&mut reporter) {
        Ok(()) => {
            println!();
            println!("✅ Batch export completed successfully!");
            println!();
            println!("All messages exported to: {}", output_path.display());
            println!();
            println!("Output structure:");
            println!("  {}/ ", output_path.display());
            println!("    00001/message.html  <- First message from first PST");
            println!("    00002/message.html  <- Second message from first PST");
            println!("    ...                 <- More messages from first PST");
            println!("    0000N/message.html  <- First message from second PST");
            println!("    ...                 <- Continues sequentially");
            println!();
            println!("Key Features Demonstrated:");
            println!("  ✓ Alphabetical PST file processing");
            println!("  ✓ Sequential numbering across all files");
            println!("  ✓ Deterministic output (same results on repeated runs)");
            println!();
            println!("Next Steps:");
            println!("  - Open any message.html file in a web browser");
            println!("  - Use --csv flag to generate spreadsheet summary");
            println!("  - Use --metadata flag to include message metadata");
            println!("  - See other examples for more features");
        }
        Err(e) => {
            eprintln!();
            eprintln!("❌ Batch export failed: {e}");
            eprintln!();
            match &e {
                pst_cli::error::Error::Pst(pst_error) => {
                    eprintln!("PST Error Details:");
                    eprintln!("  {pst_error}");
                    eprintln!();
                    eprintln!("Note: Some PST files may have been processed successfully");
                    eprintln!("before the error occurred. Check the output directory.");
                }
                pst_cli::error::Error::Io(io_error) => {
                    eprintln!("I/O Error Details:");
                    eprintln!("  {io_error}");
                    eprintln!();
                    eprintln!("Suggestions:");
                    eprintln!("  - Check directory permissions");
                    eprintln!("  - Verify output directory is writable");
                    eprintln!("  - Ensure sufficient disk space");
                }
                _ => {
                    eprintln!("Error Details:");
                    eprintln!("  {e}");
                }
            }
            process::exit(3);
        }
    }
}
