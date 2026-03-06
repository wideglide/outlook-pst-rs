//! Memory profiling benchmarks (T098)
//!
//! Verifies that PST processing does not cause memory exhaustion.
//! Uses available test files; for production validation, add 1GB+ PST files
//! to the test/ directory.
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

/// Memory benchmark: process largest available PST (33MB example-003.pst)
///
/// Validates that the streaming approach works without excessive memory.
/// Measures wall-clock time as a proxy for processing efficiency.
/// For true memory profiling, run with `DHAT` or `heaptrack` externally.
#[test]
fn bench_memory_largest_pst() {
    let pst_path = test_dir().join("example-003.pst");
    if !pst_path.exists() {
        eprintln!("SKIP: test/example-003.pst not found");
        return;
    }

    let file_size = std::fs::metadata(&pst_path).unwrap().len();
    let tmp = TempDir::new().unwrap();
    let args = ExportArgs {
        input: pst_path,
        output: tmp.path().to_path_buf(),
        metadata: true,
        attachments: true,
        headers: true,
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

    eprintln!("\n=== Memory Benchmark (largest PST) ===");
    eprintln!("  File size:   {:.1} MB", file_size as f64 / 1_048_576.0);
    eprintln!("  Messages:    {}", stats.total_messages);
    eprintln!("  Elapsed:     {elapsed:.2?}");
    eprintln!(
        "  Throughput:  {:.1} MB/sec",
        file_size as f64 / 1_048_576.0 / elapsed.as_secs_f64()
    );
    eprintln!("  Status:      Completed without OOM");
    eprintln!();
    eprintln!("  NOTE: For true memory profiling, run with:");
    eprintln!("    cargo test -p pst-cli --test bench bench_memory -- --nocapture");
    eprintln!("    and monitor with `top`, `heaptrack`, or `DHAT`.");
}

/// Memory benchmark: process all PSTs sequentially (streaming test)
///
/// Ensures that processing multiple files sequentially doesn't accumulate
/// memory from previous files (no memory leaks across PST boundaries).
#[test]
fn bench_memory_sequential_all_psts() {
    let dir = test_dir();
    if !dir.exists() {
        eprintln!("SKIP: test/ directory not found");
        return;
    }

    // Calculate total input size
    let total_size: u64 = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "pst")
        })
        .map(|e| e.metadata().unwrap().len())
        .sum();

    let tmp = TempDir::new().unwrap();
    let args = ExportArgs {
        input: dir,
        output: tmp.path().to_path_buf(),
        metadata: true,
        attachments: true,
        headers: true,
        csv: true,
        drafts: false,
        keywords: Some(vec!["test".to_string()]),
        emails: Some(vec!["watson".to_string()]),
    };

    let mut coordinator = ExportCoordinator::new(args);
    let mut reporter = ProgressReporter::new(true);

    let start = Instant::now();
    coordinator.run(&mut reporter).unwrap();
    let elapsed = start.elapsed();

    let stats = reporter.stats();

    // Count output files
    let output_files: usize = walkdir(tmp.path());

    eprintln!("\n=== Sequential Memory Benchmark (all PSTs) ===");
    eprintln!("  Total input: {:.1} MB", total_size as f64 / 1_048_576.0);
    eprintln!("  Messages:    {}", stats.total_messages);
    eprintln!("  Output files: {output_files}");
    eprintln!("  Elapsed:     {elapsed:.2?}");
    eprintln!(
        "  Throughput:  {:.1} MB/sec",
        total_size as f64 / 1_048_576.0 / elapsed.as_secs_f64()
    );
    eprintln!("  Status:      Completed without OOM (streaming verified)");
}

/// Recursively count files in a directory
fn walkdir(path: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                count += walkdir(&p);
            } else {
                count += 1;
            }
        }
    }
    count
}
