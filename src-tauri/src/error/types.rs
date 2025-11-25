//! Centralized error types for the application
//!
//! Provides structured error handling with context and user-friendly messages.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Central application error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    /// Database-related errors
    #[error("Database error: {message}")]
    Database { message: String, code: String },

    /// Cache-related errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String, field: Option<String> },

    /// Authentication/Authorization errors
    #[error("Authentication error: {message}")]
    Authentication { message: String },

    /// Analysis/prediction errors
    #[error("Analysis error: {message}")]
    Analysis { message: String },

    /// Performance errors
    #[error("Performance error: {message}")]
    Performance { message: String },

    /// IO/Filesystem errors
    #[error("IO error: {message}")]
    Io { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Internal/Unexpected errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// User input errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}

impl AppError {
    /// Get error code for programmatic handling
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database { .. } => "DB_ERROR",
            AppError::Cache { .. } => "CACHE_ERROR",
            AppError::Validation { .. } => "VALIDATION_ERROR",
            AppError::Authentication { .. } => "AUTH_ERROR",
            AppError::Analysis { .. } => "ANALYSIS_ERROR",
            AppError::Performance { .. } => "PERF_ERROR",
            AppError::Io { .. } => "IO_ERROR",
            AppError::Network { .. } => "NETWORK_ERROR",
            AppError::Configuration { .. } => "CONFIG_ERROR",
            AppError::Internal { .. } => "INTERNAL_ERROR",
            AppError::InvalidInput { .. } => "INVALID_INPUT",
            AppError::NotFound { .. } => "NOT_FOUND",
        }
    }

    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database { message, .. } => {
                format!("Database operation failed. Please try again. ({})", message)
            }
            AppError::Cache { message } => {
                format!("Cache operation failed. ({})", message)
            }
            AppError::Validation { message, field } => {
                if let Some(field) = field {
                    format!("Invalid value for {}: {}", field, message)
                } else {
                    format!("Invalid input: {}", message)
                }
            }
            AppError::Authentication { message } => {
                format!("Authentication failed: {}", message)
            }
            AppError::Analysis { message } => {
                format!("Analysis failed: {}. Please check your input parameters.", message)
            }
            AppError::Performance { message } => {
                format!("Performance issue detected: {}", message)
            }
            AppError::Io { message } => {
                format!("File operation failed: {}", message)
            }
            AppError::Network { message } => {
                format!("Network error: {}. Please check your connection.", message)
            }
            AppError::Configuration { message } => {
                format!("Configuration error: {}. Please check your settings.", message)
            }
            AppError::Internal { message } => {
                format!("An unexpected error occurred. Please try again or contact support. ({})", message)
            }
            AppError::InvalidInput { message } => {
                format!("Invalid input: {}", message)
            }
            AppError::NotFound { resource } => {
                format!("{} not found.", resource)
            }
        }
    }

    /// Get error severity for logging and monitoring
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Internal { .. } => ErrorSeverity::Critical,
            AppError::Database { .. } => ErrorSeverity::High,
            AppError::Authentication { .. } => ErrorSeverity::High,
            AppError::Configuration { .. } => ErrorSeverity::High,
            AppError::Performance { .. } => ErrorSeverity::Medium,
            AppError::Analysis { .. } => ErrorSeverity::Medium,
            AppError::Cache { .. } => ErrorSeverity::Medium,
            AppError::Network { .. } => ErrorSeverity::Medium,
            AppError::Io { .. } => ErrorSeverity::Medium,
            AppError::Validation { .. } => ErrorSeverity::Low,
            AppError::InvalidInput { .. } => ErrorSeverity::Low,
            AppError::NotFound { .. } => ErrorSeverity::Low,
        }
    }

    /// Check if error is recoverable by user action
    pub fn is_user_recoverable(&self) -> bool {
        matches!(
            self,
            AppError::Validation { .. }
                | AppError::InvalidInput { .. }
                | AppError::Cache { .. }
                | AppError::Network { .. }
        )
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Error context for better debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ErrorSeverity,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub operation: Option<String>,
    pub request_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl ErrorContext {
    pub fn new(severity: ErrorSeverity) -> Self {
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            severity,
            user_id: None,
            session_id: None,
            operation: None,
            request_id: None,
            metadata: serde_json::Value::Object(Default::default()),
        }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

/// Convert common error types to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database {
            message: err.to_string(),
            code: "SQLX_ERROR".to_string(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io {
            message: err.to_string(),
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        AppError::Performance {
            message: "Operation timed out".to_string(),
        }
    }
}