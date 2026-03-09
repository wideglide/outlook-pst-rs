//! Conversation grouping overhead benchmark.
//!
//! Run with:
//! `cargo test -p pst-cli --test bench conversations_grouping_overhead -- --nocapture`

#![allow(clippy::cast_precision_loss)]

use pst_cli::cli::progress::ProgressReporter;
use pst_cli::cli::ExportArgs;
use pst_cli::export::ExportCoordinator;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;

fn test_pst() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test/example-001.pst")
}

#[test]
fn conversations_grouping_overhead() {
    let pst_path = test_pst();
    if !pst_path.exists() {
        eprintln!("SKIP: test/example-001.pst not found");
        return;
    }

    let baseline_tmp = TempDir::new().unwrap();
    let mut baseline = ExportCoordinator::new(ExportArgs {
        input: pst_path.clone(),
        output: baseline_tmp.path().to_path_buf(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: false,
        keywords: None,
        emails: None,
    });
    let mut baseline_reporter = ProgressReporter::new(true);

    let baseline_start = Instant::now();
    baseline.run(&mut baseline_reporter).unwrap();
    let baseline_elapsed = baseline_start.elapsed();

    let grouped_tmp = TempDir::new().unwrap();
    let mut grouped = ExportCoordinator::new(ExportArgs {
        input: pst_path,
        output: grouped_tmp.path().to_path_buf(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        conversations: true,
        keywords: None,
        emails: None,
    });
    let mut grouped_reporter = ProgressReporter::new(true);

    let grouped_start = Instant::now();
    grouped.run(&mut grouped_reporter).unwrap();
    let grouped_elapsed = grouped_start.elapsed();

    let base_secs = baseline_elapsed.as_secs_f64();
    let grouped_secs = grouped_elapsed.as_secs_f64();
    let overhead_pct = if base_secs > 0.0 {
        ((grouped_secs - base_secs) / base_secs) * 100.0
    } else {
        0.0
    };

    eprintln!("\n=== Conversation Grouping Overhead ===");
    eprintln!("  Baseline:     {baseline_elapsed:.2?}");
    eprintln!("  Conversations:{grouped_elapsed:.2?}");
    eprintln!("  Overhead:     {overhead_pct:.2}%");
    eprintln!(
        "  Messages:     {}",
        grouped_reporter.stats().total_messages
    );
}
