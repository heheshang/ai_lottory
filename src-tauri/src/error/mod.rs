//! Centralized error management module
//!
//! Provides consistent error handling across the application with:
//! - Structured error types
//! - Error context tracking
//! - User-friendly error messages
//! - Error reporting and logging

pub mod types;
pub mod context;
pub mod logger;
pub mod reporting;

pub use types::*;
pub use context::ErrorContext;
pub use logger::ErrorLogger;
pub use reporting::ErrorReporter;