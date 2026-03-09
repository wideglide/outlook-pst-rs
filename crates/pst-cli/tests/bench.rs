//! Performance benchmark test harness
//!
//! Run with: `cargo test -p pst-cli --test bench -- --nocapture`

mod bench {
    mod conversations_bench;
    mod export_bench;
    mod memory_bench;
}
