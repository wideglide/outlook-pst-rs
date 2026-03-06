//! Simple usage example of pst-cli
//!
//! This example demonstrates basic usage of the export and list commands

fn main() {
    println!("=== pst-cli Usage Examples ===\n");

    println!("1. Basic PST Export:");
    println!("   $ pst-cli export archive.pst --output ./export\n");

    println!("2. Export with Metadata:");
    println!("   $ pst-cli export archive.pst --output ./export --metadata --csv\n");

    println!("3. Batch Processing:");
    println!("   $ pst-cli export ./pst_files/ --output ./batch_export\n");

    println!("4. With Filtering:");
    println!("   $ pst-cli export archive.pst --output ./export \\");
    println!("       --keywords 'confidential,merger' \\");
    println!("       --emails 'ceo@company.com,legal@company.com'\n");

    println!("5. List PST Contents:");
    println!("   $ pst-cli list archive.pst\n");

    println!("For more information, run: pst-cli --help");
}
