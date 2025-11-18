//! # Prompt Tracking System - Core Library
//!
//! Enterprise-grade prompt tracking system for Claude Code.
//!
//! ## Features
//!
//! - **Automatic Capture**: Capture prompts from files or direct input
//! - **Quality Analysis**: Score prompts on clarity, completeness, specificity, and guidance
//! - **Efficiency Metrics**: Track token usage, execution time, and cost
//! - **Reporting**: Generate reports in Markdown, HTML, JSON, and CSV formats
//! - **Version History**: Track changes and restore previous versions
//! - **Caching**: In-memory caching for improved performance
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use prompt_tracking::{
//!     database::Database,
//!     capture::CaptureService,
//!     analysis::{QualityAnalyzer, EfficiencyAnalyzer},
//! };
//!
//! // Initialize database
//! let db = Database::new("~/.prompt-tracking/prompts.db").unwrap();
//!
//! // Capture a prompt
//! let service = CaptureService::default();
//! let prompt = service.process_content("Write a function to sort an array").unwrap();
//!
//! // Save to database
//! db.create_prompt(&prompt).unwrap();
//!
//! // Analyze quality
//! let analyzer = QualityAnalyzer::default();
//! let score = analyzer.analyze(&prompt).unwrap();
//! println!("Quality Score: {:.1}", score.total_score);
//! ```

pub mod models;
pub mod database;
pub mod capture;
pub mod analysis;
pub mod reporting;
pub mod utils;
pub mod config;
pub mod watcher;
pub mod cache;

use thiserror::Error;

/// Core error types for the prompt tracking system.
///
/// This enum represents all possible errors that can occur within the
/// prompt tracking system, providing detailed context for debugging.
#[derive(Error, Debug)]
pub enum PromptTrackingError {
    /// Database operation failed
    ///
    /// This error occurs when a SQLite operation fails, such as
    /// connection issues, query errors, or constraint violations.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// File not found at specified path
    ///
    /// This error occurs when attempting to read a file that
    /// does not exist or is not accessible.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid prompt format
    ///
    /// This error occurs when the prompt content is empty or
    /// cannot be parsed correctly.
    #[error("Invalid prompt format: content is empty or malformed")]
    InvalidFormat,

    /// Configuration error
    ///
    /// This error occurs when loading or parsing configuration
    /// files fails.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Analysis error
    ///
    /// This error occurs when quality or efficiency analysis
    /// fails due to invalid input or calculation errors.
    #[error("Analysis error: {0}")]
    AnalysisError(String),

    /// I/O error
    ///
    /// This error wraps standard I/O errors for file operations,
    /// network requests, and other I/O-bound operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Cache error
    ///
    /// This error occurs when cache operations fail, such as
    /// lock acquisition failures.
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Serialization error
    ///
    /// This error occurs when JSON or YAML serialization/deserialization fails.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Version not found
    ///
    /// This error occurs when attempting to restore a prompt
    /// to a version that does not exist.
    #[error("Version {version} not found for prompt {prompt_id}")]
    VersionNotFound {
        prompt_id: String,
        version: i32,
    },

    /// Duplicate detected
    ///
    /// This error occurs when attempting to create a prompt
    /// that already exists (based on content hash).
    #[error("Duplicate prompt detected: {0}")]
    DuplicateDetected(String),
}

/// Type alias for Result with PromptTrackingError.
///
/// This is the standard result type used throughout the library
/// for operations that may fail.
pub type Result<T> = std::result::Result<T, PromptTrackingError>;

/// Library version string.
///
/// This constant contains the current version of the library
/// as defined in Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Check if the library is properly initialized.
///
/// # Returns
///
/// Always returns `true` for this simple check.
///
/// # Examples
///
/// ```
/// assert!(prompt_tracking::is_initialized());
/// ```
pub fn is_initialized() -> bool {
    true
}
