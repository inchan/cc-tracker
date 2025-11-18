//! Prompt Tracking System - Core Library
//!
//! Enterprise-grade prompt tracking system for Claude Code.
//! Provides automatic prompt capture, quality analysis, and reporting.

pub mod models;
pub mod database;
pub mod capture;
pub mod analysis;
pub mod reporting;
pub mod utils;
pub mod config;

use thiserror::Error;

/// Core error types for the prompt tracking system
#[derive(Error, Debug)]
pub enum PromptTrackingError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid prompt format")]
    InvalidFormat,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Analysis error: {0}")]
    AnalysisError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Type alias for Result with PromptTrackingError
pub type Result<T> = std::result::Result<T, PromptTrackingError>;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
