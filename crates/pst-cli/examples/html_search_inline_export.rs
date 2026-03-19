//! HTML-aware keyword search and inline attachment export example
//!
//! Demonstrates Feature 003: HTML keyword filtering (visible-text only)
//! and inline attachment reference rewriting.
//!
//! Run: `cargo run --example html_search_inline_export`

fn main() {
    println!("=== HTML Search & Inline Attachment Export ===\n");

    println!("Feature: HTML-Aware Keyword Search (US1)");
    println!("  Keywords are matched against visible text only, excluding:");
    println!("  - HTML tag names and attribute values");
    println!("  - <script> content");
    println!("  - <style> content");
    println!("  - HTML comments\n");

    println!("  Example:");
    println!("  $ pst-cli export archive.pst --output ./export \\");
    println!("      --keywords 'confidential,merger'\n");
    println!("  If a message body is HTML like:");
    println!("    <style>.confidential {{ color: red }}</style>");
    println!("    <p>The merger is proceeding.</p>");
    println!("  Only 'merger' matches (visible text), not 'confidential' (inside <style>).\n");

    println!("Feature: Inline Attachment Reference Rewriting (US2)");
    println!("  When --attachments is enabled, cid: and content-location");
    println!("  references in message.html are rewritten to point to");
    println!("  locally exported attachment files.\n");

    println!("  Example:");
    println!("  $ pst-cli export archive.pst --output ./export --attachments\n");
    println!("  Before: <img src=\"cid:image001@mail\">");
    println!("  After:  <img src=\"image001.png\">\n");

    println!("Feature: Preserve External Links (US3)");
    println!("  External URLs, mailto: links, anchors, and unresolved");
    println!("  references remain untouched during rewriting.\n");

    println!("  Preserved: https://..., mailto:..., #anchor, data:...");
    println!("  Preserved: cid: references with no matching attachment");
    println!("  Preserved: references matching multiple attachments (ambiguous)\n");

    println!("Combined Example:");
    println!("  $ pst-cli export archive.pst --output ./export \\");
    println!("      --attachments --keywords 'merger,confidential' --csv\n");

    println!("For more information, run: pst-cli --help");
}
