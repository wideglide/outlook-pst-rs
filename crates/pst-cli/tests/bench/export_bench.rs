//! Export performance benchmarks (T097)
//!
//! Measures export throughput and extrapolates to 1K/10K message targets.
//! The performance goal from SC-009: export 10,000 messages in <10 minutes.
//!
//! Run: `cargo test -p pst-cli --test bench -- --nocapture`

#![allow(clippy::cast_precision_loss)]

use pst_cli::cli::ExportArgs;
use pst_cli::cli::progress::ProgressReporter;
use pst_cli::export::ExportCoordinator;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;

/// Find the test PST directory (repo root/test/)
fn test_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().parent().unwrap().join("test")
}

/// Find a specific test PST file
fn test_pst(name: &str) -> PathBuf {
    test_dir().join(name)
}

/// Benchmark: single PST export (basic HTML only)
#[test]
fn bench_basic_export_single_pst() {
    let pst_path = test_pst("example-001.pst");
    if !pst_path.exists() {
        eprintln!("SKIP: test/example-001.pst not found");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let args = ExportArgs {
        input: pst_path,
        output: tmp.path().to_path_buf(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        keywords: None,
        emails: None,
    };

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true); // quiet mode

    let start = Instant::now();
    coordinator.run(&mut reporter).unwrap();
    let elapsed = start.elapsed();

    let stats = reporter.stats();
    let msg_count = stats.total_messages;
    let per_msg_ms = if msg_count > 0 {
        elapsed.as_millis() as f64 / msg_count as f64
    } else {
        0.0
    };

    eprintln!("\n=== Basic Export Benchmark (single PST) ===");
    eprintln!("  Messages: {msg_count}");
    eprintln!("  Elapsed:  {elapsed:.2?}");
    eprintln!("  Per msg:  {per_msg_ms:.2} ms");
    eprintln!("  Extrapolated 1K:  {:.1} sec", per_msg_ms * 1_000.0 / 1_000.0);
    eprintln!("  Extrapolated 10K: {:.1} sec", per_msg_ms * 10_000.0 / 1_000.0);

    let ten_k_seconds = per_msg_ms * 10_000.0 / 1_000.0;
    eprintln!(
        "  10K target (<600s): {} ({ten_k_seconds:.1}s)",
        if ten_k_seconds < 600.0 {
            "PASS"
        } else {
            "FAIL"
        }
    );
}

/// Benchmark: full export with all flags enabled
#[test]
fn bench_full_export_all_flags() {
    let pst_path = test_pst("example-001.pst");
    if !pst_path.exists() {
        eprintln!("SKIP: test/example-001.pst not found");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let args = ExportArgs {
        input: pst_path,
        output: tmp.path().to_path_buf(),
        metadata: true,
        attachments: true,
        headers: true,
        csv: true,
        drafts: false,
        keywords: Some(vec!["test".to_string(), "email".to_string()]),
        emails: Some(vec!["watson".to_string()]),
    };

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let start = Instant::now();
    coordinator.run(&mut reporter).unwrap();
    let elapsed = start.elapsed();

    let stats = reporter.stats();
    let msg_count = stats.total_messages;
    let per_msg_ms = if msg_count > 0 {
        elapsed.as_millis() as f64 / msg_count as f64
    } else {
        0.0
    };

    eprintln!("\n=== Full Export Benchmark (all flags) ===");
    eprintln!("  Messages:    {msg_count}");
    eprintln!("  Duplicates:  {}", stats.duplicates);
    eprintln!("  Elapsed:     {elapsed:.2?}");
    eprintln!("  Per msg:     {per_msg_ms:.2} ms");
    eprintln!("  Extrapolated 1K:  {:.1} sec", per_msg_ms * 1_000.0 / 1_000.0);
    eprintln!("  Extrapolated 10K: {:.1} sec", per_msg_ms * 10_000.0 / 1_000.0);

    let ten_k_seconds = per_msg_ms * 10_000.0 / 1_000.0;
    eprintln!(
        "  10K target (<600s): {} ({ten_k_seconds:.1}s)",
        if ten_k_seconds < 600.0 {
            "PASS"
        } else {
            "FAIL"
        }
    );
}

/// Benchmark: batch export across all test PST files
#[test]
fn bench_batch_export() {
    let dir = test_dir();
    if !dir.exists() {
        eprintln!("SKIP: test/ directory not found");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let args = ExportArgs {
        input: dir,
        output: tmp.path().to_path_buf(),
        metadata: true,
        attachments: true,
        headers: false,
        csv: true,
        drafts: false,
        keywords: None,
        emails: None,
    };

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let start = Instant::now();
    coordinator.run(&mut reporter).unwrap();
    let elapsed = start.elapsed();

    let stats = reporter.stats();
    let msg_count = stats.total_messages;
    let per_msg_ms = if msg_count > 0 {
        elapsed.as_millis() as f64 / msg_count as f64
    } else {
        0.0
    };

    eprintln!("\n=== Batch Export Benchmark (all PSTs) ===");
    eprintln!("  Messages:    {msg_count}");
    eprintln!("  Duplicates:  {}", stats.duplicates);
    eprintln!("  Elapsed:     {elapsed:.2?}");
    eprintln!("  Per msg:     {per_msg_ms:.2} ms");
    eprintln!(
        "  Throughput:  {:.1} msgs/sec",
        if per_msg_ms > 0.0 {
            1_000.0 / per_msg_ms
        } else {
            0.0
        }
    );
}

/// Benchmark: duplicate detection overhead
///
/// Compares single-file export against batch (which naturally generates
/// cross-file duplicate checks).
#[test]
fn bench_duplicate_detection_overhead() {
    let pst_path = test_pst("example-001.pst");
    if !pst_path.exists() {
        eprintln!("SKIP: test/example-001.pst not found");
        return;
    }

    // First pass: baseline (single file, no cross-file duplicates)
    let tmp1 = TempDir::new().unwrap();
    let args1 = ExportArgs {
        input: pst_path.clone(),
        output: tmp1.path().to_path_buf(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        keywords: None,
        emails: None,
    };
    let mut coord1 = ExportCoordinator::new(args1);
    let mut rep1 = ProgressReporter::new(true);
    let start1 = Instant::now();
    coord1.run(&mut rep1).unwrap();
    let elapsed1 = start1.elapsed();

    // Second pass: batch with all PSTs (triggers dedup across files)
    let tmp2 = TempDir::new().unwrap();
    let args2 = ExportArgs {
        input: test_dir(),
        output: tmp2.path().to_path_buf(),
        metadata: false,
        attachments: false,
        headers: false,
        csv: false,
        drafts: false,
        keywords: None,
        emails: None,
    };
    let mut coord2 = ExportCoordinator::new(args2);
    let mut rep2 = ProgressReporter::new(true);
    let start2 = Instant::now();
    coord2.run(&mut rep2).unwrap();
    let elapsed2 = start2.elapsed();

    let stats2 = rep2.stats();

    eprintln!("\n=== Duplicate Detection Benchmark ===");
    eprintln!(
        "  Single file:    {elapsed1:.2?} ({} msgs)",
        rep1.stats().total_messages
    );
    eprintln!(
        "  Batch (3 PSTs): {elapsed2:.2?} ({} msgs, {} dups)",
        stats2.total_messages, stats2.duplicates
    );

    // Use variables to avoid unused warnings
    let _ = pst_path;
}
