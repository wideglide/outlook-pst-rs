//! Command-line entry point for pst-cli
use pst_cli::cli::{Cli, Command};
use pst_cli::export::ExportCoordinator;
use pst_cli::list::ListCommand;
use clap::Parser;

fn main() -> pst_cli::error::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle subcommand
    match cli.command {
        Command::Export(export_args) => {
            let mut reporter = pst_cli::cli::progress::ProgressReporter::new(cli.quiet);
            reporter.set_include_drafts_in_export(export_args.drafts);
            
            let mut coordinator = ExportCoordinator::new(export_args);
            coordinator.run(&mut reporter)?;
            
            // Display summary statistics (unless --quiet)
            reporter.summary_statistics();
            
            Ok(())
        }
        Command::List(list_args) => {
            let cmd = ListCommand::new(list_args);
            cmd.run()?;
            
            Ok(())
        }
    }
}
