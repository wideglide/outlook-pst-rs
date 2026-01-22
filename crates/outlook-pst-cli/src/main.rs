mod args;
mod benchmark;
mod encoding;
mod iterate_emails;

use anyhow::Result;
use clap::Parser;

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// List emails and details in the console
    List(args::ListArgs),
    /// Export emails (HTML + attachments) to a directory
    Export(args::ExportArgs),
    /// Benchmark recipient/attachment enumeration performance
    Bench(args::BenchArgs),
}

#[derive(Parser, Debug)]
#[command(
    name = "outlook-pst-cli",
    version,
    about = "CLI utilities for Outlook PST files"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::List(args) => iterate_emails::run_list(args),
        Command::Export(args) => iterate_emails::run_export(args),
        Command::Bench(args) => benchmark::run_bench(args),
    }
}
