//! Filtering logic for keywords and email participants

pub mod email;
pub mod keyword;

// Re-export for convenience
pub use email::EmailMatcher;
pub use keyword::KeywordMatcher;
