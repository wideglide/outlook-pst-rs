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

    /// Force enumerating the MAPI recipient table even if headers contain recipients
    #[arg(long)]
    pub full_recipients: bool,

    /// Extra diagnostics when a message lacks a recipient table (logs row metadata from the contents table)
    #[arg(long, default_value_t = false)]
    pub debug_missing_rt: bool,
}

/// Arguments for the `export` subcommand
#[derive(Parser, Clone, Debug)]
#[command(about = "Export emails as HTML (and attachments) to a directory", long_about = None)]
pub struct ExportArgs {
    /// Path to a PST file or a directory containing PST files
    #[clap(default_value = r#"crates/pst/Example-001.pst"#)]
    pub input: String,

    /// Output directory to write each email as an HTML file
    #[arg(long)]
    pub out_dir: String,

    /// Also save attachments for each message into the same index folder as message.html
    #[arg(long)]
    pub attachments: bool,

    /// Write message metadata (non-transport headers) into metadata.txt next to the HTML
    #[arg(long)]
    pub metadata: bool,

    /// Write transport headers into headers.txt when present
    #[arg(long)]
    pub headers: bool,

    /// Also write a CSV summary of all emails to the root of the output directory
    #[arg(long)]
    pub csv: bool,

    /// Comma-separated list of email addresses; if any participate in an email, list them as "Responsive Emails"
    #[arg(long, value_delimiter = ',', value_name = "EMAILS")]
    pub responsive_emails: Vec<String>,

    /// Comma-separated list of keywords; if any are found in the email body, list them as "Keywords"
    #[arg(long, value_delimiter = ',', value_name = "KEYWORDS")]
    pub keywords: Vec<String>,

    /// Force enumerating the MAPI recipient table even if headers contain recipients
    #[arg(long)]
    pub full_recipients: bool,

    /// Extra diagnostics when a message lacks a recipient table (logs row metadata from the contents table)
    #[arg(long, default_value_t = false)]
    pub debug_missing_rt: bool,
}

/// Arguments for the `bench` subcommand (recipient/attachment enumeration performance)
#[derive(Parser, Clone, Debug)]
#[command(about = "Benchmark recipient table usage and timing", long_about = None)]
pub struct BenchArgs {
    /// Path to a PST file or a directory containing PST files
    #[clap(default_value = r#"crates/pst/Example-001.pst"#)]
    pub input: String,

    /// Number of warm-up passes (ignored for now, reserved)
    #[arg(long, default_value_t = 0)]
    pub warmups: u32,

    /// Emit CSV with per-pass aggregate stats to stdout
    #[arg(long)]
    pub csv: bool,

    /// Force always reading recipient table even when headers supply recipients
    #[arg(long)]
    pub force_table: bool,

    /// Perform a lazy fallback read of recipient table only when headers have no recipients (default true; disable to measure header-only speed)
    #[arg(long, default_value_t = true)]
    pub lazy_fallback: bool,

    /// Also measure attachment table enumeration (currently stubbed if not implemented)
    #[arg(long)]
    pub attachments: bool,
}
