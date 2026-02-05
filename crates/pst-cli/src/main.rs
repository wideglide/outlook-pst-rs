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
            
            let mut coordinator = ExportCoordinator::new(export_args);
            coordinator.run()?;
            
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
