use clap::Parser;

/// Arguments for the `list` subcommand
#[derive(Parser, Clone, Debug)]
#[command(about = "List emails and details in the console", long_about = None)]
pub struct ListArgs {
    /// Path to a PST file or a directory containing PST files
    #[clap(default_value = r#"crates/pst/Example-001.pst"#)]
    pub input: String,

    /// Show internet transport headers (if present) for each message
    #[arg(long)]
    pub show_headers: bool,

    /// Show a list of attachments for each message
    #[arg(long)]
    pub show_attachments: bool,

    /// Show which body types are present (text, html, rtf)
    #[arg(long)]
    pub show_body_types: bool,

    /// Comma-separated list of email addresses; if any participate in an email, list them as "Responsive Emails"
    #[arg(long, value_delimiter = ',', value_name = "EMAILS")]
    pub responsive_emails: Vec<String>,

    /// Comma-separated list of keywords; if any are found in the email body, list them as "Keywords"
    #[arg(long, value_delimiter = ',', value_name = "KEYWORDS")]
    pub keywords: Vec<String>,

    /// Also write a CSV summary of all emails to the current working directory
    #[arg(long)]
    pub csv: bool,
}

/// Arguments for the `dump` subcommand
#[derive(Parser, Clone, Debug)]
#[command(about = "Dump HTML-formatted emails to a directory", long_about = None)]
pub struct DumpArgs {
    /// Path to a PST file or a directory containing PST files
    #[clap(default_value = r#"crates/pst/Example-001.pst"#)]
    pub input: String,

    /// Output directory to write each email as an HTML file
    #[arg(long)]
    pub out_dir: String,

    /// Also save attachments for each message into the same index folder as message.html
    #[arg(long)]
    pub attachments: bool,

    /// Also write a CSV summary of all emails to the root of the output directory
    #[arg(long)]
    pub csv: bool,

    /// Comma-separated list of email addresses; if any participate in an email, list them as "Responsive Emails"
    #[arg(long, value_delimiter = ',', value_name = "EMAILS")]
    pub responsive_emails: Vec<String>,

    /// Comma-separated list of keywords; if any are found in the email body, list them as "Keywords"
    #[arg(long, value_delimiter = ',', value_name = "KEYWORDS")]
    pub keywords: Vec<String>,

    /// Write message metadata (Subject, From, Date, To, Cc, Bcc, MessageId, Folder, Size, Flags, etc.) to metadata.txt
    #[arg(long)]
    pub metadata: bool,

    /// Write transport headers (Received, X-Mailer, etc.) to headers.txt when available
    #[arg(long)]
    pub headers: bool,
}
