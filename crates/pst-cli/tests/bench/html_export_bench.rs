//! HTML export benchmarks (T028)
//!
//! Measures performance of visible-text extraction and inline-reference
//! rewriting for HTML bodies of varying sizes.
//!
//! Run: `cargo test -p pst-cli --test bench -- html_export --nocapture`

#![allow(clippy::cast_precision_loss)]

use pst_cli::export::exporter::AttachmentExportPlanEntry;
use pst_cli::export::html::{extract_visible_text, rewrite_inline_references};
use std::time::Instant;

/// Generate a realistic HTML body of approximately `n` paragraphs.
fn generate_html(paragraphs: usize, inline_images: usize) -> String {
    let mut html = String::from("<html><head><style>body { font-family: Calibri; }</style></head><body>");
    for i in 0..paragraphs {
        html.push_str(&format!(
            "<p>Paragraph {i} with some representative email text about quarterly results and upcoming meetings.</p>"
        ));
        if i < inline_images {
            html.push_str(&format!(r#"<img src="cid:image{i}@mail">"#));
        }
    }
    html.push_str("<script>var tracking = true;</script></body></html>");
    html
}

fn make_plan_entries(count: usize) -> Vec<AttachmentExportPlanEntry> {
    (0..count)
        .map(|i| AttachmentExportPlanEntry {
            attachment_index: i,
            resolved_filename: format!("image{i}.png"),
            relative_path: format!("image{i}.png"),
            content_id_keys: vec![format!("image{i}@mail")],
            content_location_keys: vec![],
        })
        .collect()
}

#[test]
fn bench_visible_text_extraction_small() {
    let html = generate_html(10, 0);
    let iterations = 1_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = extract_visible_text(&html);
    }
    let elapsed = start.elapsed();

    let per_op = elapsed / iterations;
    eprintln!(
        "visible_text_extraction (10 paragraphs, {} bytes): {:?}/op ({} ops/sec)",
        html.len(),
        per_op,
        1_000_000 / per_op.as_micros().max(1)
    );
}

#[test]
fn bench_visible_text_extraction_large() {
    let html = generate_html(200, 0);
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = extract_visible_text(&html);
    }
    let elapsed = start.elapsed();

    let per_op = elapsed / iterations;
    eprintln!(
        "visible_text_extraction (200 paragraphs, {} bytes): {:?}/op ({} ops/sec)",
        html.len(),
        per_op,
        1_000_000 / per_op.as_micros().max(1)
    );
}

#[test]
fn bench_inline_rewrite_small() {
    let html = generate_html(10, 5);
    let entries = make_plan_entries(5);
    let iterations = 1_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = rewrite_inline_references(&html, &entries);
    }
    let elapsed = start.elapsed();

    let per_op = elapsed / iterations;
    eprintln!(
        "inline_rewrite (10 paragraphs, 5 images, {} bytes): {:?}/op ({} ops/sec)",
        html.len(),
        per_op,
        1_000_000 / per_op.as_micros().max(1)
    );
}

#[test]
fn bench_inline_rewrite_large() {
    let html = generate_html(200, 50);
    let entries = make_plan_entries(50);
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = rewrite_inline_references(&html, &entries);
    }
    let elapsed = start.elapsed();

    let per_op = elapsed / iterations;
    eprintln!(
        "inline_rewrite (200 paragraphs, 50 images, {} bytes): {:?}/op ({} ops/sec)",
        html.len(),
        per_op,
        1_000_000 / per_op.as_micros().max(1)
    );
}
