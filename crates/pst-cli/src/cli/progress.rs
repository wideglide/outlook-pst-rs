//! Progress reporting and summary statistics
//!
//! Provides progress indicators for long-running operations and summary
//! statistics display (unless --quiet flag is used).

use std::io::{self, Write};
use std::time::Instant;

/// Progress reporter for export operations
#[derive(Debug)]
pub struct ProgressReporter {
    /// Start time of the operation
    start_time: Instant,
    /// Whether quiet mode is enabled
    quiet: bool,
    /// Statistics tracking
    stats: ExportStatistics,
}

/// Export operation statistics
#[derive(Debug, Default)]
pub struct ExportStatistics {
    /// Total messages processed
    pub total_messages: usize,
    /// Number of duplicates found
    pub duplicates: usize,
    /// Number of errors encountered
    pub errors: usize,
    /// Number of keyword matches
    pub keyword_matches: usize,
    /// Number of email participant matches
    pub email_matches: usize,
    /// Elapsed time in seconds
    pub elapsed_secs: u64,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(quiet: bool) -> Self {
        Self {
            start_time: Instant::now(),
            quiet,
            stats: ExportStatistics::default(),
        }
    }

    /// Update progress indicator (prints to stderr)
    pub fn update_progress(&self, current: usize, total: usize) {
        if self.quiet {
            return;
        }

        // Print progress counter to stderr (updated in-place)
        eprint!("\rProcessing message {}/{}...", current, total);
        let _ = io::stderr().flush();
    }

    /// Record a duplicate message
    pub fn record_duplicate(&mut self) {
        self.stats.duplicates += 1;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.stats.errors += 1;
    }

    /// Record a keyword match
    pub fn record_keyword_match(&mut self) {
        self.stats.keyword_matches += 1;
    }

    /// Record an email participant match
    pub fn record_email_match(&mut self) {
        self.stats.email_matches += 1;
    }

    /// Set total messages processed
    pub fn set_total_messages(&mut self, count: usize) {
        self.stats.total_messages = count;
    }

    /// Display summary statistics (unless quiet mode)
    pub fn summary_statistics(&mut self) {
        if self.quiet {
            return;
        }

        // Calculate elapsed time
        self.stats.elapsed_secs = self.start_time.elapsed().as_secs();

        // Clear progress line
        eprintln!();

        // Display summary table
        eprintln!("\n=== Export Summary ===");
        eprintln!("Total messages:      {}", self.stats.total_messages);
        eprintln!("Duplicates found:    {}", self.stats.duplicates);
        eprintln!("Errors encountered:  {}", self.stats.errors);

        if self.stats.keyword_matches > 0 {
            eprintln!("Keyword matches:     {}", self.stats.keyword_matches);
        }

        if self.stats.email_matches > 0 {
            eprintln!("Email matches:       {}", self.stats.email_matches);
        }

        eprintln!("Elapsed time:        {}s", self.stats.elapsed_secs);
        eprintln!("======================\n");
    }

    /// Get current statistics
    pub fn stats(&self) -> &ExportStatistics {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_quiet() {
        let mut reporter = ProgressReporter::new(true);
        
        // These should not panic or output anything
        reporter.update_progress(5, 10);
        reporter.record_duplicate();
        reporter.record_error();
        reporter.set_total_messages(10);
        reporter.summary_statistics();

        assert_eq!(reporter.stats().total_messages, 10);
        assert_eq!(reporter.stats().duplicates, 1);
        assert_eq!(reporter.stats().errors, 1);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut reporter = ProgressReporter::new(true);

        reporter.set_total_messages(100);
        reporter.record_duplicate();
        reporter.record_duplicate();
        reporter.record_error();
        reporter.record_keyword_match();
        reporter.record_keyword_match();
        reporter.record_keyword_match();
        reporter.record_email_match();

        let stats = reporter.stats();
        assert_eq!(stats.total_messages, 100);
        assert_eq!(stats.duplicates, 2);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.keyword_matches, 3);
        assert_eq!(stats.email_matches, 1);
    }
}
