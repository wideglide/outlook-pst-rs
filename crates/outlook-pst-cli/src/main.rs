mod args;
mod encoding;
mod iterate_emails;

use anyhow::Result;
use clap::Parser;

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// List emails and details in the console
    List(args::ListArgs),
    /// Dump HTML-formatted emails to a directory
    Dump(args::DumpArgs),
}

#[derive(Parser, Debug)]
#[command(name = "outlook-pst-cli", version, about = "CLI utilities for Outlook PST files")] 
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
    Command::List(args) => iterate_emails::run_list(args),
    Command::Dump(args) => iterate_emails::run_dump(args),
    }
}
